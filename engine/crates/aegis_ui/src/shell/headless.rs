//! A launch with no window (HEADLESS1): the same screens, `Shell::frame` and agent link at a chosen size, the frames left
//! in the shell's software canvas. No `winit` window and no GPU surface exist, so it runs on a machine with no display
//! or audio device. Selected by `Options::headless`; a game reads it from its launch arguments (`Headless::from_args`).
//!
//! The pace is `pacing::Pacer`'s, with a low cap: a screen at rest draws at its own need and a resting one takes next to
//! no CPU, and the link never forces a frame (in a window every request does). Time on screen is wall time, as in a window;
//! nothing here fakes a clock. Two link requests exist only here: `frames` (frames now) and `clip` (a numbered sequence).
//! With no window to close, a run also ends when nothing has asked anything for a while (`Headless::idle`): a driver that
//! was killed leaves no invisible game behind.

use super::{blank_reason, save_png, text_scale_of, Options, Shell};
use crate::bridge;
use crate::pacing::{Pace, Pacer};
use crate::screen::{window_size_of, Context, Cx, Screen};
use anyhow::{bail, Context as _, Result};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::mpsc::RecvTimeoutError;
use std::time::{Duration, Instant};

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
/// A clip's bounds: frames, time between frames and the whole clip as asked (`clip` holds the link's other requests until it
/// is done); then what writing may stretch it to, and what it may put on disk. Past either it stops and fails.
pub const CLIP_FRAMES_MAX: u64 = 120;
pub const CLIP_EVERY_MS: (u64, u64) = (10, 2000);
pub const CLIP_MS_MAX: u64 = 15_000;
pub const CLIP_WALL_MS_MAX: u64 = 30_000;
pub const CLIP_BYTES_MAX: u64 = 256 << 20;
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
}

impl Headless {
    /// From launch arguments: `headless=WxH`, with `fps=` (default `DEFAULT_FPS`), `dpr=` (default 1) and `idle_exit=`
    /// (seconds, default `IDLE_EXIT_S`). None without `headless=`. A value out of bounds, or one of the three alone, is
    /// refused.
    pub fn from_args(args: &[(String, String)]) -> Result<Option<Self>> {
        let arg = |k: &str| args.iter().find(|(a, _)| a == k).map(|(_, v)| v.as_str());
        let Some(text) = arg("headless") else {
            if ["fps", "dpr", "idle_exit"].iter().any(|k| arg(k).is_some()) {
                bail!("fps=, dpr= and idle_exit= come with headless=WxH");
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
        Ok(Some(Self { size, fps, dpr, idle: Duration::from_secs(idle_s) }))
    }
}

struct Loop<G> {
    shell: Shell<G>,
    pacer: Pacer,
    every: Duration,
    /// The settings last applied (None: not yet).
    applied: Option<Value>,
    /// When the last link request was done, or the run started.
    asked_at: Instant,
}

impl<G: Context> Loop<G> {
    /// One frame, as a window's redraw: the shell's own step, then the settings. What a window does with the OS (the
    /// pointer, the surface) has no place here.
    fn draw(&mut self) {
        let started = Instant::now();
        self.shell.frame();
        self.shell.pointer_req = None;
        self.apply_settings();
        self.pacer.framed(started, self.shell.frame_need());
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
        self.pacer.touch(Instant::now());
        let reply = match req.body["cmd"].as_str() {
            Some("frames") => self.frames(&req.body),
            Some("clip") => self.clip(&req.body),
            _ => self.shell.bridge(&req.body),
        };
        let _ = req.reply.send(reply);
        self.asked_at = Instant::now();
    }

    /// `{"cmd": "frames", "frames": n}`, default 1: draws them now, before the reply, so what the last request changed is on
    /// the canvas with no waiting for the pace. Clocks read wall time, so this does not move time on. Anything else a screen
    /// answers (a game's own `step`) goes through to it: a name here hides one of a screen's.
    fn frames(&mut self, body: &Value) -> Value {
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

    /// `{"cmd": "clip", "dir": ..., "frames": n, "every_ms": m, "max_w": w}`: a numbered sequence (`frame_0001.png` on) of
    /// the screen as it runs, replying when it is done. Frames go on at the pace between two saves, so clocks stay on wall
    /// time. A blank frame leaves a gap in the numbers and its number under `blank`; a clip of nothing but those fails.
    fn clip(&mut self, body: &Value) -> Value {
        let plan = match ClipPlan::read(body, self.every) {
            Ok(p) => p,
            Err(e) => return json!({"ok": false, "error": e}),
        };
        let started = Instant::now();
        let (mut saved, mut blank, mut ms, mut bytes) = (Vec::new(), Vec::new(), Vec::new(), 0u64);
        for i in 0..plan.frames {
            let due = started + plan.every * i as u32;
            while Instant::now() + self.every < due {
                std::thread::sleep(self.every);
                self.draw();
            }
            std::thread::sleep(due.saturating_duration_since(Instant::now()));
            self.draw();
            ms.push(started.elapsed().as_millis() as u64);
            if blank_reason(&self.shell.gfx.canvas).is_some() {
                blank.push(i + 1);
                if let Some(why) = clip_over(bytes, started.elapsed()) {
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
            if let Some(why) = clip_over(bytes, started.elapsed()) {
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
struct ClipPlan {
    dir: PathBuf,
    frames: u64,
    every: Duration,
    max_w: i32,
}

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
fn clip_over(bytes: u64, took: Duration) -> Option<String> {
    if bytes > CLIP_BYTES_MAX {
        return Some(format!("{bytes} bytes written, at most {CLIP_BYTES_MAX}"));
    }
    (took > Duration::from_millis(CLIP_WALL_MS_MAX)).then(|| format!("{} ms taken, at most {CLIP_WALL_MS_MAX}", took.as_millis()))
}

/// An earlier clip's frames would pass for this one's, gaps and all.
fn holds_frames(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else { return false };
    entries.flatten().any(|e| e.file_name().to_string_lossy().starts_with("frame_"))
}

/// Runs the screens with no window until a screen or the link sets `cx.quit`: then they leave and this returns, so the
/// game exits as a window's does. The link's port being taken is an error here, with no window to show it on. No link
/// request for `Headless::idle` is an error too, after the screens leave: its driver is gone, and nothing else can end this.
pub(super) fn run<G: Context>(cx: Cx<G>, stack: Vec<Box<dyn Screen<G>>>, options: &Options, h: Headless) -> Result<()> {
    // The same notice over the first screen as a window, so the two walk the same screens.
    let icon_notice = crate::icon::window_icons(cx.art.layers(), &options.icon).err();
    let Some(link) = bridge::start(options.port, || {}) else { bail!("Agent link: port {} is busy.", options.port) };
    let shell = Shell::open(cx, stack, options, h.size, h.dpr, icon_notice);
    let every = Duration::from_micros(1_000_000 / u64::from(h.fps));
    let mut game = Loop { shell, pacer: Pacer::default(), every, applied: None, asked_at: Instant::now() };
    loop {
        if let Ok(req) = link.try_recv() {
            game.serve(req);
        }
        if !game.shell.cx.quit {
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
        if game.asked_at.elapsed() >= h.idle {
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
