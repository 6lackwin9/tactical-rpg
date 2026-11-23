//! A launch with no window (HEADLESS1): the same screens, `Shell::frame` and agent link at a chosen size, the frames left
//! in the shell's software canvas. No `winit` window and no GPU surface exist, so it runs on a machine with no display
//! or audio device. Selected by `Options::headless`; a game reads it from its launch arguments (`Headless::from_args`).
//!
//! The pace is `pacing::Pacer`'s, with a low cap: a screen at rest draws at its own need and a resting one takes next to
//! no CPU, and the link never forces a frame (in a window every request does). Time on screen is wall time, as in a window,
//! unless the launch asks for the shot clock (`clock=step`, ENGINE-17, which every scenario run with no window does):
//! then the process clock is stepped (`aegis_core::clock::step_on`), no frame is drawn at the pace, and each frame a
//! request draws moves time on by one frame step (1 / fps) first, so a run's frames, and its shots, are the same in every
//! run of the build. Two link requests exist only here: `frames` (frames now) and `clip` (a numbered sequence).
//! With no window to close, a run also ends when nothing has asked anything for a while (`Headless::idle`): a driver that
//! was killed leaves no invisible game behind.
//!
//! Native only (WEBREL1-E1b): the run loop needs the agent link (TCP, threads), `std::thread::sleep` and clip files,
//! none of which the web build has. `Headless` and its argument parsing stay on every target; on the web `run` refuses.

use super::Options;
use crate::screen::{Context, Cx, Key, Screen};
use aegis_core::clock::Duration;
use anyhow::{bail, Context as _, Result};
#[cfg(not(target_arch = "wasm32"))]
use {
    super::{blank_reason, save_png, settings::text_scale_of, Shell},
    crate::bridge,
    crate::pacing::{Pace, Pacer},
    crate::screen::window_size_of,
    aegis_core::clock::Instant,
    serde_json::{json, Value},
    std::path::{Path, PathBuf},
    std::sync::mpsc::RecvTimeoutError,
};

/// ENGINE-48: how far around a widget its focus mark may be drawn, in px.
pub const REACH_MARGIN_PX: i32 = 8;

/// ENGINE-48: whether two frames (width, height, RGBA) of one size differ inside `r` (px, cut to the canvas).
#[cfg(not(target_arch = "wasm32"))]
fn region_differs(a: &(i32, i32, Vec<u8>), b: &(i32, i32, Vec<u8>), r: crate::canvas::Rect) -> bool {
    let (w, h) = (a.0, a.1);
    if (w, h) != (b.0, b.1) {
        return true;
    }
    let (x0, y0, x1, y1) = (r.x.max(0), r.y.max(0), (r.x + r.w).min(w), (r.y + r.h).min(h));
    (y0..y1).any(|y| {
        let (i, j) = (((y * w + x0) * 4) as usize, ((y * w + x1.max(x0)) * 4) as usize);
        a.2[i..j] != b.2[i..j]
    })
}

/// The default frame rate.
pub const DEFAULT_FPS: u32 = 15;
/// The lowest asked for: a clock that limits how much wall time one frame adds (a common guard against a stall) must not
/// reach that limit at the pace alone, with the sleep's overshoot on top, or animations fall behind wall time.
pub const FPS_MIN: u32 = 12;
pub const FPS_MAX: u32 = 240;
/// A run ends when no link request has come for this long, in seconds, unless `idle_exit=` says otherwise.
pub const IDLE_EXIT_S: u64 = 120;
pub const IDLE_EXIT_RANGE_S: (u64, u64) = (1, 86_400);
/// The sizes taken: the smallest a window has, and the largest a scenario takes.
pub const SIZE_MIN: (u32, u32) = (640, 360);
pub const SIZE_MAX: (u32, u32) = (7680, 4320);
pub const DPR_RANGE: (f32, f32) = (0.5, 4.0);
/// The most frames one `frames` request draws.
pub const FRAMES_MAX: u64 = 240;
/// The most shot-clock time one `frames` `ms` or `since_open_ms` request moves on (ENGINE-17).
pub const STEP_MS_MAX: u64 = 120_000;
/// A clip's bounds: frames, time between frames and the whole clip as asked (`clip` holds the link's other requests until it
/// is done); then what writing may stretch it to, and what it may put on disk. Past either it stops and fails.
pub const CLIP_FRAMES_MAX: u64 = 120;
pub const CLIP_EVERY_MS: (u64, u64) = (10, 2000);
pub const CLIP_MS_MAX: u64 = 15_000;
pub const CLIP_WALL_MS_MAX: u64 = 30_000;
pub const CLIP_BYTES_MAX: u64 = 256 << 20;
/// A `perf` request's bounds (ENGINE-2): the frames timed and the frames drawn first, untimed.
pub const PERF_FRAMES_MAX: u64 = 2000;
pub const PERF_WARM_MAX: u64 = 600;
/// A `feel` request's frames between two inputs by default, and the most `times` one script input may ask for.
pub const FEEL_GAP_DEFAULT: u64 = 2;
pub const FEEL_TIMES_MAX: u64 = 200;
#[cfg(not(target_arch = "wasm32"))]
const CLIP_MAX_W: u64 = 1280;

/// How a headless run draws.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Headless {
    /// The canvas, in physical px.
    pub size: (u32, u32),
    pub fps: u32,
    /// The device's px per logical px (a window takes it from the OS); 1 is the same on every machine.
    pub dpr: f32,
    /// How long the run goes on with no link request.
    pub idle: Duration,
    /// `clock=step`: the shot clock (time moves only by the frames requests draw).
    pub stepped: bool,
}

impl Headless {
    /// From launch arguments: `headless=WxH`, with `fps=` (default `DEFAULT_FPS`), `dpr=` (default 1) and `idle_exit=`
    /// (seconds, default `IDLE_EXIT_S`). None without `headless=`. A value out of bounds, or one of the three alone, is
    /// refused.
    pub fn from_args(args: &[(String, String)]) -> Result<Option<Self>> {
        let arg = |k: &str| args.iter().find(|(a, _)| a == k).map(|(_, v)| v.as_str());
        let Some(text) = arg("headless") else {
            if ["fps", "dpr", "idle_exit", "clock"].iter().any(|k| arg(k).is_some()) {
                bail!("fps=, dpr=, idle_exit= and clock= come with headless=WxH");
            }
            return Ok(None);
        };
        let size = text.split_once('x').and_then(|(w, h)| Some((w.parse::<u32>().ok()?, h.parse::<u32>().ok()?))).with_context(|| format!("headless='{text}' is not WxH"))?;
        if size.0 < SIZE_MIN.0 || size.1 < SIZE_MIN.1 || size.0 > SIZE_MAX.0 || size.1 > SIZE_MAX.1 {
            bail!("headless='{text}' is outside {}x{} to {}x{}", SIZE_MIN.0, SIZE_MIN.1, SIZE_MAX.0, SIZE_MAX.1);
        }
        let fps = match arg("fps") {
            None => DEFAULT_FPS,
            Some(t) => t.parse().ok().filter(|n| (FPS_MIN..=FPS_MAX).contains(n)).with_context(|| format!("fps='{t}' is not a whole number from {FPS_MIN} to {FPS_MAX}"))?,
        };
        let dpr = match arg("dpr") {
            None => 1.0,
            Some(t) => t.parse().ok().filter(|d: &f32| (DPR_RANGE.0..=DPR_RANGE.1).contains(d)).with_context(|| format!("dpr='{t}' is not a number from {} to {}", DPR_RANGE.0, DPR_RANGE.1))?,
        };
        let idle_s = match arg("idle_exit") {
            None => IDLE_EXIT_S,
            Some(t) => t.parse().ok().filter(|s| (IDLE_EXIT_RANGE_S.0..=IDLE_EXIT_RANGE_S.1).contains(s)).with_context(|| format!("idle_exit='{t}' is not a whole number of seconds from {} to {}", IDLE_EXIT_RANGE_S.0, IDLE_EXIT_RANGE_S.1))?,
        };
        let stepped = match arg("clock") {
            None | Some("wall") => false,
            Some("step") => true,
            Some(t) => bail!("clock='{t}' is wall or step"),
        };
        Ok(Some(Self { size, fps, dpr, idle: Duration::from_secs(idle_s), stepped }))
    }
}

/// ENGINE-49: what a `feel` replay with `motion` records: its start (the run's clock), every frame's motion and every
/// input.
#[cfg(not(target_arch = "wasm32"))]
struct MotionRec {
    t0: Instant,
    frames: Vec<crate::walkfeel::Frame>,
    marks: Vec<crate::walkfeel::Mark>,
}

#[cfg(not(target_arch = "wasm32"))]
struct Loop<G> {
    shell: Shell<G>,
    pacer: Pacer,
    every: Duration,
    /// The settings last applied (None: not yet).
    applied: Option<Value>,
    /// When the last link request was done, or the run started (wall time).
    asked_at: Instant,
    /// The shot clock is on (`Headless::stepped`).
    stepped: bool,
    /// The top screen (stack depth and id) and when it opened, on the run's clock: `frames` `since_open_ms` reads it.
    top: (usize, &'static str),
    opened: Instant,
}

#[cfg(not(target_arch = "wasm32"))]
impl<G: Context> Loop<G> {
    /// One frame, as a window's redraw: the shell's own step, then the settings. What a window does with the OS (the
    /// pointer, the surface) has no place here.
    fn draw(&mut self) {
        if self.stepped {
            aegis_core::clock::advance(self.every);
        }
        let started = Instant::wall();
        let feel_start = self.shell.feel_now();
        self.shell.frame();
        // ENGINE-45: nothing is presented with no window; the frame counts as shown once drawn
        self.shell.feel_framed(feel_start);
        self.shell.pointer_req = None;
        self.apply_settings();
        self.pacer.framed(started, self.shell.frame_need());
        self.note_top();
    }

    /// Notes when the top screen changed (a push, a pop or a swap), on the run's clock.
    fn note_top(&mut self) {
        let top = (self.shell.stack.len(), self.shell.stack.last().map_or("", |s| s.id()));
        if top != self.top {
            self.top = top;
            self.opened = Instant::now();
        }
    }

    /// The text size, and a window size picked in the game's settings: the canvas follows as a window would. The launch's
    /// size stands over what the profile says at the start.
    fn apply_settings(&mut self) {
        let shell = &mut self.shell;
        if self.applied.as_ref() == Some(&shell.cx.settings.v) {
            return;
        }
        let v = shell.cx.settings.v.clone();
        if self.applied.as_ref().is_some_and(|a| a["window_size"] != v["window_size"]) {
            if let Some((w, h)) = window_size_of(&v["window_size"]).filter(|s| s.0 >= SIZE_MIN.0 && s.1 >= SIZE_MIN.1 && s.0 <= SIZE_MAX.0 && s.1 <= SIZE_MAX.1) {
                shell.resize_canvas(w, h);
            }
        }
        shell.gfx.fonts.scale = text_scale_of(&v);
        self.applied = Some(v);
    }

    /// A request, answered at once whatever the pace; it counts as a change a frame must show, as it does in a window.
    fn serve(&mut self, req: bridge::Request) {
        self.pacer.touch(Instant::wall());
        let reply = match req.body["cmd"].as_str() {
            Some("frames") => self.frames(&req.body),
            Some("clip") => self.clip(&req.body),
            Some("perf") => self.perf(&req.body),
            Some("feel") => self.feel(&req.body),
            Some("reach") => self.reach(&req.body),
            _ => self.shell.bridge(&req.body),
        };
        let _ = req.reply.send(reply);
        self.note_top();
        self.asked_at = Instant::wall();
    }

    /// `{"cmd": "frames", "frames": n}`, default 1: draws them now, before the reply, so what the last request changed is on
    /// the canvas with no waiting for the pace. On wall time this does not move time on; on the shot clock each frame moves
    /// it one frame step. Two more forms (ENGINE-17), which move the shot clock on and draw a frame at each step: `ms`
    /// (draws until that much time has passed) and `since_open_ms` (until the top screen has been open that long; none
    /// when it already has). Their time is capped at `STEP_MS_MAX`; on wall time they are refused. Anything else a screen
    /// answers (a game's own `step`) goes through to it: a name here hides one of a screen's.
    fn frames(&mut self, body: &Value) -> Value {
        for key in ["ms", "since_open_ms"] {
            let Some(v) = body.get(key) else { continue };
            let Some(ms) = v.as_u64().filter(|n| *n <= STEP_MS_MAX) else {
                return json!({"ok": false, "error": format!("{key} is a whole number from 0 to {STEP_MS_MAX}")});
            };
            if !self.stepped {
                return json!({"ok": false, "error": format!("{key} needs the shot clock (clock=step)")});
            }
            let until = if key == "ms" { Instant::now() } else { self.opened } + Duration::from_millis(ms);
            let mut drawn = 0u64;
            while Instant::now() < until {
                self.draw();
                drawn += 1;
            }
            return json!({"ok": true, "frame": self.shell.frames, "drawn": drawn});
        }
        let frames = match body.get("frames") {
            None => 1,
            Some(n) => match n.as_u64().filter(|n| (1..=FRAMES_MAX).contains(n)) {
                Some(n) => n,
                None => return json!({"ok": false, "error": format!("frames is a whole number from 1 to {FRAMES_MAX}")}),
            },
        };
        for _ in 0..frames {
            self.draw();
        }
        json!({"ok": true, "frame": self.shell.frames})
    }

    /// ENGINE-2 (PERF-STEP1): `{"cmd": "perf", "frames": n, "warm": w}`, defaults 300 and 60: draws `warm` frames untimed,
    /// then times `frames` whole frames (`Shell::frame`, all a frame costs with no window: nothing is presented) one after
    /// another, with the calibration workload (`perf::calib`) before and after. Replies with the frame numbers
    /// (`perf::Stats`, ms), both calibrations, `steady` (`perf::steady`) and the canvas. Back to back, not at the pace:
    /// the numbers are the frame's cost, not the frame rate.
    fn perf(&mut self, body: &Value) -> Value {
        let whole = |key: &str, default: u64, (low, high): (u64, u64)| match body.get(key) {
            None => Ok(default),
            Some(n) => n.as_u64().filter(|n| (low..=high).contains(n)).ok_or(format!("{key} is a whole number from {low} to {high}")),
        };
        let (frames, warm) = match (whole("frames", 300, (1, PERF_FRAMES_MAX)), whole("warm", 60, (0, PERF_WARM_MAX))) {
            (Ok(f), Ok(w)) => (f, w),
            (Err(e), _) | (_, Err(e)) => return json!({"ok": false, "error": e}),
        };
        for _ in 0..warm {
            self.draw();
        }
        let calib_before = crate::perf::calib();
        let mut ms = Vec::with_capacity(frames as usize);
        for _ in 0..frames {
            let t = Instant::wall();
            self.draw();
            ms.push(crate::perf::ms_since(t));
        }
        let calib_after = crate::perf::calib();
        let r = |x: f64| (x * 1000.0).round() / 1000.0;
        let canvas = [self.shell.gfx.canvas.w, self.shell.gfx.canvas.h];
        json!({"ok": true, "stats": crate::perf::Stats::of(&ms).to_json(), "calib_before": r(calib_before), "calib_after": r(calib_after),
            "steady": crate::perf::steady(calib_before, calib_after), "canvas": canvas, "build": if cfg!(debug_assertions) { "dev" } else { "release" }, "frame": self.shell.frames})
    }

    /// ENGINE-45 (FEEL-PROBE1): `{"cmd": "feel", "script": [...], "gap": g}`: replays the script's inputs one by one, each
    /// timed by the feel probe (`crate::feel`) from its arrival through its handler to the end of the next frame (drawn:
    /// nothing is presented with no window), then `gap` more frames (default 2) before the next. An input is
    /// `{"key": name}`, `{"click": widget id}`, `{"pad": button}` or `{"frames": n}` (frames drawn, no input), each with an
    /// optional `"times"`. Replies with `feel::report` (p50, p95 and worst of input-to-state, input-to-present and frame
    /// time, every input, the flagged ones), the canvas and the build. A refused input stops the replay with its error.
    fn feel(&mut self, body: &Value) -> Value {
        let Some(script) = body["script"].as_array() else { return json!({"ok": false, "error": "feel needs script: a list of {key|click|pad|frames}"}) };
        let gap = match body.get("gap") {
            None => FEEL_GAP_DEFAULT,
            Some(g) => match g.as_u64().filter(|g| *g <= FRAMES_MAX) {
                Some(g) => g,
                None => return json!({"ok": false, "error": format!("gap is a whole number from 0 to {FRAMES_MAX}")}),
            },
        };
        let was_on = self.shell.feel.on;
        let was_launch = self.shell.feel_launch;
        self.shell.feel_launch = true;
        self.shell.feel.on = true;
        let _ = self.shell.feel.take();
        // ENGINE-49 (WALK-FEEL1): `step_ms` sets the shot clock's frame step for this replay; `motion` reads the top
        // screen's `motion` after every frame for the movement report (`crate::walkfeel`)
        let every = self.every;
        if let Some(ms) = body["step_ms"].as_f64() {
            if !(1.0..=200.0).contains(&ms) || !self.stepped {
                return json!({"ok": false, "error": "step_ms is from 1 to 200, with the shot clock on (a scenario run)"});
            }
            self.every = Duration::from_secs_f64(ms / 1000.0);
        }
        let mut rec = (body["motion"] == json!(true)).then(|| MotionRec { t0: Instant::now(), frames: Vec::new(), marks: Vec::new() });
        // where things stand before the first input (the last frame drawn)
        if let Some(r) = rec.as_mut() {
            r.frames.extend(crate::walkfeel::Frame::of(0.0, &self.shell.stack.last().unwrap().motion()));
        }
        let mut held: Vec<Key> = Vec::new();
        let mut done = 0u64;
        let mut failed = None;
        'script: for (i, step) in script.iter().enumerate() {
            let times = step["times"].as_u64().unwrap_or(1).clamp(1, FEEL_TIMES_MAX);
            let dir_key = |name: Option<&str>| name.and_then(Key::parse).filter(|k| k.dir().is_some());
            let pad_key = |name: Option<&str>, shell: &mut Shell<G>| name.and_then(crate::controls::Pad::parse).map(|p| shell.controls().pad(p)).filter(|k| k.dir().is_some());
            // a held direction: pressed (as a window's key down, kept held) or let go
            let hold = match (step["press"].as_str(), step["release"].as_str(), step["pad_press"].as_str(), step["pad_release"].as_str()) {
                (Some(n), _, _, _) => Some(("press", n, dir_key(Some(n)), true)),
                (_, Some(n), _, _) => Some(("release", n, dir_key(Some(n)), false)),
                (_, _, Some(n), _) => Some(("pad_press", n, pad_key(Some(n), &mut self.shell), true)),
                (_, _, _, Some(n)) => Some(("pad_release", n, pad_key(Some(n), &mut self.shell), false)),
                _ => None,
            };
            if let Some((kind, name, k, down)) = hold {
                let Some(k) = k else {
                    failed = Some(format!("script[{i}] {kind}: '{name}' is no movement key or d-pad direction"));
                    break;
                };
                let before = !held.is_empty();
                held.retain(|h| *h != k);
                if down {
                    held.push(k);
                }
                self.shell.cx.held.dir = held.last().and_then(|h| h.dir());
                let what = format!("{kind} {name}");
                self.mark(&mut rec, kind, &what, k.dir(), before, !held.is_empty());
                if down {
                    self.shell.feel_input(&what);
                    self.shell.key(k);
                    self.shell.feel_state();
                }
                done += 1;
                for _ in 0..=gap {
                    self.feel_draw(&mut rec);
                }
                continue;
            }
            if let Some(ms) = step["wait_ms"].as_f64() {
                let until = Instant::now() + Duration::from_secs_f64(ms.clamp(0.0, 60_000.0) / 1000.0);
                let mut n = 0;
                while Instant::now() < until && n < 60 * 60 {
                    self.feel_draw(&mut rec);
                    n += 1;
                }
                continue;
            }
            if let Some(ms) = step["spike_ms"].as_f64() {
                // a frame that takes `ms` (the shot clock moves that much more before it)
                let held_now = !held.is_empty();
                self.mark(&mut rec, "spike", &format!("spike {ms} ms"), None, held_now, held_now);
                if self.stepped {
                    aegis_core::clock::advance(Duration::from_secs_f64(ms.clamp(0.0, 2000.0) / 1000.0).saturating_sub(self.every));
                }
                self.feel_draw(&mut rec);
                continue;
            }
            let (cmd, kind, mark) = if let Some(k) = step["key"].as_str() {
                (json!({"cmd": "key", "key": k}), format!("key {k}"), ("key", Key::parse(k).and_then(|k| k.dir())))
            } else if let Some(id) = step["click"].as_str() {
                (json!({"cmd": "click", "id": id}), format!("click {id}"), ("click", None))
            } else if let Some(b) = step["pad"].as_str() {
                (json!({"cmd": "pad", "button": b}), format!("pad {b}"), ("pad", pad_key(Some(b), &mut self.shell).and_then(|k| k.dir())))
            } else if let Some(n) = step["frames"].as_u64() {
                for _ in 0..n.min(FRAMES_MAX) * times {
                    self.feel_draw(&mut rec);
                }
                continue;
            } else {
                failed = Some(format!("script[{i}]: one of key, click, pad, frames, press, release, pad_press, pad_release, wait_ms or spike_ms"));
                break;
            };
            for _ in 0..times {
                self.mark(&mut rec, mark.0, &kind, mark.1, !held.is_empty(), !held.is_empty());
                self.shell.feel_input(&kind);
                let reply = self.shell.bridge(&cmd);
                self.shell.feel_state();
                if reply["ok"] != json!(true) {
                    failed = Some(format!("script[{i}] {kind}: {}", reply["error"]));
                    break 'script;
                }
                done += 1;
                for _ in 0..=gap {
                    self.feel_draw(&mut rec);
                }
            }
        }
        self.shell.cx.held.dir = None;
        self.every = every;
        let (samples, frames) = self.shell.feel.take();
        self.shell.feel_launch = was_launch;
        self.shell.feel.on = was_on;
        if let Some(e) = failed {
            return json!({"ok": false, "error": e, "done": done});
        }
        let mut out = crate::feel::report(&samples, &frames);
        out["ok"] = json!(true);
        out["canvas"] = json!([self.shell.gfx.canvas.w, self.shell.gfx.canvas.h]);
        out["build"] = json!(if cfg!(debug_assertions) { "dev" } else { "release" });
        out["frame"] = json!(self.shell.frames);
        if let Some(r) = rec {
            out["motion"] = crate::walkfeel::report(&r.frames, &r.marks, body["step_ms"].as_f64().unwrap_or(every.as_secs_f64() * 1000.0));
        }
        out
    }

    /// ENGINE-49: a frame of a `feel` replay, and the top screen's `motion` after it when the replay records it.
    fn feel_draw(&mut self, rec: &mut Option<MotionRec>) {
        self.draw();
        if let Some(r) = rec.as_mut() {
            let t = Instant::now().saturating_duration_since(r.t0).as_secs_f64() * 1000.0;
            if let Some(f) = crate::walkfeel::Frame::of(t, &self.shell.stack.last().unwrap().motion()) {
                r.frames.push(f);
            }
        }
    }

    /// ENGINE-49: notes an input of a `feel` replay for the movement report.
    fn mark(&self, rec: &mut Option<MotionRec>, kind: &str, what: &str, dir: Option<(i64, i64)>, held_before: bool, held_after: bool) {
        if let Some(r) = rec.as_mut() {
            let t = Instant::now().saturating_duration_since(r.t0).as_secs_f64() * 1000.0;
            r.marks.push(crate::walkfeel::Mark { t, kind: kind.into(), what: what.into(), dir, held_before, held_after });
        }
    }

    /// ENGINE-48 (INPUT-REACH1): `{"cmd": "reach", "back": b}`: the top screen's reach report (`crate::reach::report`:
    /// what a mouse can use that the keys, the d-pad alone or a pad cannot reach), then whether the focus shows on each
    /// widget of the ring (a frame with it against one without, around the widget: `invisible` when nothing changed,
    /// `unsure` when the screen moves there by itself), then with `back` (default false) one Esc through the control map
    /// and what it did: `closed` (a screen left), `opened` (one came over it), `swapped`, `changed` (the same screen with
    /// other widgets), `nothing` or `quits` (the game asked to quit: the run is kept going). The focus is put back after
    /// the focus check; the Esc is not undone.
    fn reach(&mut self, body: &Value) -> Value {
        use crate::reach;
        let shell = &mut self.shell;
        let top = shell.stack.last().unwrap();
        let (screen, infos, arrows, grid, group) = (top.id(), top.widgets(), top.arrow_focus(), top.grid(), top.focus_group());
        let start = top.focus_first().map(String::from).or_else(|| shell.gfx.kit.focus.clone());
        let pad_keys: Vec<crate::screen::Key> = crate::controls::PADS.iter().map(|(p, _)| shell.controls().pad(*p)).collect();
        let mut out = reach::report_grouped(&infos, arrows, start.as_deref(), reach::Device::pad(&pad_keys), group);
        // the focus check: frames with the focus shown and hidden, the mouse away
        let saved = (shell.gfx.kit.focus.clone(), shell.gfx.kit.focus_visible, shell.mouse);
        shell.mouse = (-10_000, -10_000);
        let view = shell.view_of(grid);
        let frame = |shell: &mut Shell<G>, focus: Option<&str>| {
            shell.gfx.kit.focus = focus.map(String::from);
            shell.gfx.kit.focus_visible = focus.is_some();
            shell.frame();
            (shell.gfx.canvas.w, shell.gfx.canvas.h, shell.gfx.canvas.px.clone())
        };
        let base = frame(shell, None);
        let again = frame(shell, None);
        let (mut invisible, mut unsure) = (Vec::new(), Vec::new());
        for w in crate::focus::ring(&infos, false) {
            let wid = w["id"].as_str().unwrap_or("");
            let r = |i: usize| w["rect"][i].as_i64().unwrap_or(0) as i32;
            let px = view.rect(crate::canvas::Rect::new(r(0), r(1), r(2), r(3)));
            let around = crate::canvas::Rect::new(px.x - REACH_MARGIN_PX, px.y - REACH_MARGIN_PX, px.w + 2 * REACH_MARGIN_PX, px.h + 2 * REACH_MARGIN_PX);
            let shown = frame(shell, Some(wid));
            if !region_differs(&base, &shown, around) {
                invisible.push(json!({"id": wid, "kind": w["kind"]}));
            } else if region_differs(&base, &again, around) {
                unsure.push(json!({"id": wid, "kind": w["kind"]}));
            }
        }
        (shell.gfx.kit.focus, shell.gfx.kit.focus_visible, shell.mouse) = saved;
        out["invisible"] = json!(invisible);
        out["unsure"] = json!(unsure);
        if body["back"] == json!(true) {
            let before = (shell.stack.len(), screen, infos.iter().map(|w| w["id"].clone()).collect::<Vec<_>>());
            shell.press(crate::screen::Key::Esc);
            self.draw();
            let shell = &mut self.shell;
            let quits = std::mem::take(&mut shell.cx.quit);
            let top = shell.stack.last().unwrap();
            let after = (shell.stack.len(), top.id(), top.widgets().iter().map(|w| w["id"].clone()).collect::<Vec<_>>());
            let did = if quits {
                "quits"
            } else if after.0 < before.0 {
                "closed"
            } else if after.0 > before.0 {
                "opened"
            } else if after.1 != before.1 {
                "swapped"
            } else if after.2 != before.2 {
                "changed"
            } else {
                "nothing"
            };
            out["back"] = json!({"did": did, "to": after.1});
        }
        out["screen"] = json!(screen);
        out["ok"] = json!(true);
        out
    }

    /// `{"cmd": "clip", "dir": ..., "frames": n, "every_ms": m, "max_w": w}`: a numbered sequence (`frame_0001.png` on) of
    /// the screen as it runs, replying when it is done. Frames go on at the pace between two saves, so clocks stay on wall
    /// time. A blank frame leaves a gap in the numbers and its number under `blank`; a clip of nothing but those fails.
    fn clip(&mut self, body: &Value) -> Value {
        let plan = match ClipPlan::read(body, self.every) {
            Ok(p) => p,
            Err(e) => return json!({"ok": false, "error": e}),
        };
        let started = Instant::now();
        let wall = Instant::wall();
        let (mut saved, mut blank, mut ms, mut bytes) = (Vec::new(), Vec::new(), Vec::new(), 0u64);
        for i in 0..plan.frames {
            let due = started + plan.every * i as u32;
            if self.stepped {
                // The shot clock: frames at each step up to the due time, then the one saved; no sleeping.
                while Instant::now() + self.every < due {
                    self.draw();
                }
            } else {
                while Instant::now() + self.every < due {
                    std::thread::sleep(self.every);
                    self.draw();
                }
                std::thread::sleep(due.saturating_duration_since(Instant::now()));
            }
            self.draw();
            ms.push(started.elapsed().as_millis() as u64);
            if blank_reason(&self.shell.gfx.canvas).is_some() {
                blank.push(i + 1);
                if let Some(why) = clip_over(bytes, wall.wall_elapsed()) {
                    return json!({"ok": false, "error": format!("the clip stopped at frame {}: {why}", i + 1), "saved": saved});
                }
                continue;
            }
            let path = plan.dir.join(format!("frame_{:04}.png", i + 1));
            if let Err(e) = save_png(&self.shell.gfx.canvas, &path, plan.max_w) {
                return json!({"ok": false, "error": format!("{}: {e}", path.display())});
            }
            bytes += std::fs::metadata(&path).map_or(0, |m| m.len());
            saved.push(path.display().to_string());
            if let Some(why) = clip_over(bytes, wall.wall_elapsed()) {
                return json!({"ok": false, "error": format!("the clip stopped at frame {}: {why}", i + 1), "saved": saved});
            }
        }
        if saved.is_empty() {
            return json!({"ok": false, "error": format!("no clip: all {} frames were blank", plan.frames)});
        }
        json!({"ok": true, "dir": plan.dir.display().to_string(), "saved": saved, "blank": blank, "ms": ms, "frame": self.shell.frames})
    }
}

/// A `clip` request, checked.
#[cfg(not(target_arch = "wasm32"))]
struct ClipPlan {
    dir: PathBuf,
    frames: u64,
    every: Duration,
    max_w: i32,
}

#[cfg(not(target_arch = "wasm32"))]
impl ClipPlan {
    fn read(body: &Value, frame: Duration) -> std::result::Result<Self, String> {
        let dir = body["dir"].as_str().filter(|d| !d.is_empty()).map(PathBuf::from).ok_or("clip needs dir, a folder")?;
        let whole = |key: &str, default: u64, (low, high): (u64, u64)| match body.get(key) {
            None => Ok(default),
            Some(n) => n.as_u64().filter(|n| (low..=high).contains(n)).ok_or(format!("{key} is a whole number from {low} to {high}")),
        };
        let frames = whole("frames", 30, (1, CLIP_FRAMES_MAX))?;
        let every_ms = whole("every_ms", frame.as_millis() as u64, CLIP_EVERY_MS)?;
        let max_w = whole("max_w", CLIP_MAX_W, (1, u64::from(SIZE_MAX.0)))?;
        if frames.saturating_mul(every_ms) > CLIP_MS_MAX {
            return Err(format!("a clip of {frames} frames every {every_ms} ms is over {CLIP_MS_MAX} ms"));
        }
        if holds_frames(&dir) {
            return Err(format!("{} already holds a clip's frames", dir.display()));
        }
        Ok(Self { dir, frames, every: Duration::from_millis(every_ms), max_w: max_w as i32 })
    }
}

/// What is past a clip's limits, if anything: large frames written slowly or at length must not run away.
#[cfg(not(target_arch = "wasm32"))]
fn clip_over(bytes: u64, took: Duration) -> Option<String> {
    if bytes > CLIP_BYTES_MAX {
        return Some(format!("{bytes} bytes written, at most {CLIP_BYTES_MAX}"));
    }
    (took > Duration::from_millis(CLIP_WALL_MS_MAX)).then(|| format!("{} ms taken, at most {CLIP_WALL_MS_MAX}", took.as_millis()))
}

/// An earlier clip's frames would pass for this one's, gaps and all.
#[cfg(not(target_arch = "wasm32"))]
fn holds_frames(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else { return false };
    entries.flatten().any(|e| e.file_name().to_string_lossy().starts_with("frame_"))
}

/// Runs the screens with no window until a screen or the link sets `cx.quit`: then they leave and this returns, so the
/// game exits as a window's does. The link's port being taken is an error here, with no window to show it on. No link
/// request for `Headless::idle` is an error too, after the screens leave: its driver is gone, and nothing else can end this.
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn run<G: Context>(cx: Cx<G>, stack: Vec<Box<dyn Screen<G>>>, options: &Options, h: Headless) -> Result<()> {
    // The same notice over the first screen as a window, so the two walk the same screens.
    let icon_notice = crate::icon::window_icons(cx.art.layers(), &options.icon).err();
    let Some(link) = bridge::start(options.port, || {}) else { bail!("Agent link: port {} is busy.", options.port) };
    if h.stepped {
        // Before the first screen opens, so everything it stamps is on the shot clock.
        aegis_core::clock::step_on();
    }
    let shell = Shell::open(cx, stack, options, h.size, h.dpr, icon_notice);
    let every = Duration::from_micros(1_000_000 / u64::from(h.fps));
    let mut game = Loop { shell, pacer: Pacer::default(), every, applied: None, asked_at: Instant::wall(), stepped: h.stepped, top: (0, ""), opened: Instant::now() };
    if h.stepped {
        // One frame at the start, as the pace would draw: the first `ui_tree` has a screen to read.
        game.draw();
    }
    loop {
        if let Ok(req) = link.try_recv() {
            game.serve(req);
        }
        if game.stepped && !game.shell.cx.quit {
            // No pace: frames come only with the requests that ask for them.
            match link.recv_timeout(Duration::from_millis(250)) {
                Ok(req) => game.serve(req),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => std::thread::sleep(Duration::from_millis(250)),
            }
        } else if !game.shell.cx.quit {
            let now = Instant::now();
            match game.pacer.decide(now, Some(every)) {
                Pace::Redraw => game.draw(),
                Pace::WaitUntil(at) => match link.recv_timeout(at.saturating_duration_since(now)) {
                    Ok(req) => game.serve(req),
                    Err(RecvTimeoutError::Timeout) => {}
                    // No listener any more: only the pace wakes the loop, and it must still sleep.
                    Err(RecvTimeoutError::Disconnected) => std::thread::sleep(at.saturating_duration_since(now)),
                },
            }
        }
        if game.shell.cx.quit {
            game.shell.leave_all();
            return Ok(());
        }
        if game.asked_at.wall_elapsed() >= h.idle {
            // A request already waiting means its driver is still there.
            if let Ok(req) = link.try_recv() {
                game.serve(req);
                continue;
            }
            game.shell.leave_all();
            bail!("no link request for {} s: leaving (idle_exit= sets the limit)", h.idle.as_secs());
        }
    }
}

/// The web build: no headless runs (no agent link, no thread sleep, no clip files).
#[cfg(target_arch = "wasm32")]
pub(super) fn run<G: Context>(_cx: Cx<G>, _stack: Vec<Box<dyn Screen<G>>>, _options: &Options, _h: Headless) -> Result<()> {
    bail!("headless= runs are native only (no agent link on the web)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_clip_stops_past_its_disk_and_time_limits() {
        let (bytes, wall) = (CLIP_BYTES_MAX, Duration::from_millis(CLIP_WALL_MS_MAX));
        assert_eq!(clip_over(bytes, wall), None, "at the limits");
        assert!(clip_over(bytes + 1, wall).is_some_and(|w| w.contains("bytes")));
        assert!(clip_over(bytes, wall + Duration::from_millis(1)).is_some_and(|w| w.contains("ms")));
    }
}
