//! ENGINE-45 (FEEL-PROBE1): how long an input takes to reach the screen. The shell notes each input when it arrives
//! (`input`), when its handler has changed the state (`state`), and when the next frame that started after it has been
//! presented (`frame`: drawn and handed to the renderer in a window; drawn in a run with no window, which presents
//! nothing). A frame's own time is its start to its present. Nothing here changes what the game does: the probe only
//! reads the clock, and the shell keeps it off unless the launch says `--feel` (or `feel=1`; the run's switch, read
//! once by `start::Start` into `Options::feel`, ENGINE-66) or the settings say `"feel": true` (the player's switch: it
//! stays on across runs until removed; no Settings row writes it).
//!
//! On, the shell draws a small overlay (`overlay_text`: the last frame's ms, the worst frame of the last second and the
//! last input's input-to-present ms) and adds a line per input to `feel.log` beside the run's log (`log_line`). The
//! agent link's `feel` request (no window) replays a script of keys, clicks and pad buttons and replies with `report`.
//! Times are ms on the wall clock from the probe's start, so the shot clock of a scenario run does not bend them.
//!
//! ENGINE-60 (PLAN-START2): a frame over `FRAME_FLAG_MS` that spent less than `STALL_CPU_SHARE` of its wall time on the
//! CPU (the caller gives the frame's CPU time, `Probe::cpu`, where the platform tells it) is a stall: the machine
//! stopped the thread, not the frame's own work. A stall is kept apart (`take_stalls`): it is in no frame or input
//! statistic of the report, and the inputs it showed are listed with `"stall": true`.

use crate::perf::Stats;
use serde_json::{json, Value};
use std::collections::VecDeque;

/// An input whose input-to-present passes this is flagged (the row's bar for a hands-on playtest).
pub const INPUT_FLAG_MS: f64 = 100.0;
/// A frame over this is flagged (under 30 frames a second).
pub const FRAME_FLAG_MS: f64 = 33.0;
/// ENGINE-60: a frame over `FRAME_FLAG_MS` that ran on the CPU for less than this share of its wall time is a stall.
pub const STALL_CPU_SHARE: f64 = 0.5;
/// The window the overlay's worst frame is taken over, in ms.
pub const WORST_WINDOW_MS: f64 = 1000.0;
/// Samples kept for a report (the oldest go first).
pub const SAMPLES_MAX: usize = 4096;
/// The log file's name, beside the run's log (`crashlog::dir`).
pub const LOG_FILE: &str = "feel.log";
/// The overlay's text size, in the whole window's units.
pub const OVERLAY_SIZE: f32 = 7.0;

/// Whether the process's command line asks for the probe: `--feel` or `feel=1` (none on the web). `start::Start` reads
/// it with the parsed launch keys into `Options::feel`.
pub fn launch_asked() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    return std::env::args().skip(1).any(|a| a == "--feel" || a == "feel=1");
    #[cfg(target_arch = "wasm32")]
    false
}

/// Frames kept for the worst-of-the-last-second read.
const FRAMES_KEPT: usize = 512;
/// Inputs waiting for a frame: a flood past this drops the oldest (a frame always comes, so it never fills in use).
const PENDING_MAX: usize = 256;

/// An input waiting for its frame.
#[derive(Debug, Clone, PartialEq)]
struct Pending {
    kind: String,
    at: f64,
    state: Option<f64>,
}

/// One input's times, in ms.
#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    /// What it was: `key right`, `click battle.confirm`, `pad a`, `mouse left`, `wheel`.
    pub kind: String,
    /// Arrival to the handler done (the state changed).
    pub to_state: f64,
    /// Arrival to the start of the frame that shows it (the wait for the frame loop).
    pub to_frame: f64,
    /// Arrival to that frame presented.
    pub to_present: f64,
    /// That frame's own time (start to present), and the parts of it the top screens took: their `tick` and their
    /// `draw` (the rest is the shell's own work and the present).
    pub frame: f64,
    pub tick: f64,
    pub draw: f64,
    /// ENGINE-60: that frame was a stall (kept out of the report's statistics).
    pub stall: bool,
}

/// ENGINE-60: a stalled frame: its start and end (probe ms), its wall time and its CPU time (ms).
#[derive(Debug, Clone, PartialEq)]
pub struct Stall {
    pub start: f64,
    pub end: f64,
    pub ms: f64,
    pub cpu: f64,
}

impl Stall {
    pub fn to_json(&self) -> Value {
        let r = |x: f64| crate::perf::round_to(x, 3);
        json!({"start_ms": r(self.start), "frame_ms": r(self.ms), "cpu_ms": r(self.cpu)})
    }
}

/// ENGINE-60: whether a frame of `ms` wall time that ran `cpu` ms on the CPU is a stall.
pub fn is_stall(ms: f64, cpu: f64) -> bool {
    ms > FRAME_FLAG_MS && cpu < ms * STALL_CPU_SHARE
}

impl Sample {
    /// Where the time went: the largest of the handler, the wait for the frame and the frame itself.
    pub fn spent_in(&self) -> &'static str {
        let wait = (self.to_frame - self.to_state).max(0.0);
        if self.to_state >= wait && self.to_state >= self.frame {
            "handler"
        } else if wait >= self.frame {
            "wait"
        } else {
            "frame"
        }
    }

    /// Over `INPUT_FLAG_MS`.
    pub fn flagged(&self) -> bool {
        self.to_present > INPUT_FLAG_MS
    }

    pub fn to_json(&self) -> Value {
        let r = |x: f64| crate::perf::round_to(x, 3);
        json!({"input": self.kind, "to_state_ms": r(self.to_state), "to_frame_ms": r(self.to_frame), "to_present_ms": r(self.to_present), "frame_ms": r(self.frame),
            "tick_ms": r(self.tick), "draw_ms": r(self.draw), "spent_in": self.spent_in(), "flag": self.flagged(), "stall": self.stall})
    }
}

/// `feel.log`'s line for `s`: tab-separated, the run's seconds first.
pub fn log_line(now_ms: f64, s: &Sample) -> String {
    format!("{:.3}\t{}\tstate {:.1} ms\tframe start {:.1} ms\tpresent {:.1} ms\tframe {:.1} ms (tick {:.1}, draw {:.1})\t{}{}", now_ms / 1000.0, s.kind, s.to_state, s.to_frame, s.to_present, s.frame, s.tick, s.draw, s.spent_in(), if s.flagged() { "\tOVER" } else { "" })
}

/// The probe (see the module). Off, every call does nothing.
#[derive(Debug, Default)]
pub struct Probe {
    pub on: bool,
    pending: Vec<Pending>,
    /// (end, ms) of recent frames.
    frames: VecDeque<(f64, f64)>,
    /// Every frame time since the last `take` (for a report), and every input sample.
    frame_ms: Vec<f64>,
    samples: Vec<Sample>,
    /// The last input's sample, for the overlay.
    pub last: Option<Sample>,
    /// Lines for `feel.log` not written yet (the shell writes them).
    pub lines: Vec<String>,
    /// The frame being drawn: ms its screens' `tick` and `draw` took (the shell sets them; `frame` reads them).
    pub tick: f64,
    pub draw: f64,
    /// ENGINE-60: the frame being drawn's CPU time in ms, where the platform tells it (the caller sets it; `frame`
    /// takes it).
    pub cpu: Option<f64>,
    stalls: Vec<Stall>,
}

impl Probe {
    /// An input of `kind` arrived at `at`.
    pub fn input(&mut self, kind: &str, at: f64) {
        if !self.on {
            return;
        }
        if self.pending.len() >= PENDING_MAX {
            self.pending.remove(0);
        }
        self.pending.push(Pending { kind: kind.to_string(), at, state: None });
    }

    /// The handler of the latest input is done at `at`.
    pub fn state(&mut self, at: f64) {
        if let Some(p) = self.pending.iter_mut().rev().find(|p| p.state.is_none()) {
            p.state = Some(at);
        }
    }

    /// A frame that started at `start` was presented at `presented`: every input that arrived before its start is done.
    pub fn frame(&mut self, start: f64, presented: f64) {
        if !self.on {
            return;
        }
        let ms = (presented - start).max(0.0);
        if self.frames.len() >= FRAMES_KEPT {
            self.frames.pop_front();
        }
        self.frames.push_back((presented, ms));
        let stall = self.cpu.take().filter(|c| is_stall(ms, *c));
        match stall {
            Some(cpu) if self.stalls.len() < SAMPLES_MAX => self.stalls.push(Stall { start, end: presented, ms, cpu }),
            Some(_) => {}
            None if self.frame_ms.len() < SAMPLES_MAX => self.frame_ms.push(ms),
            None => {}
        }
        let (done, wait): (Vec<Pending>, Vec<Pending>) = self.pending.drain(..).partition(|p| p.at <= start);
        self.pending = wait;
        for p in done {
            let s = Sample { kind: p.kind, to_state: (p.state.unwrap_or(p.at) - p.at).max(0.0), to_frame: (start - p.at).max(0.0), to_present: (presented - p.at).max(0.0), frame: ms, tick: self.tick, draw: self.draw, stall: stall.is_some() };
            self.lines.push(log_line(presented, &s));
            if self.samples.len() >= SAMPLES_MAX {
                self.samples.remove(0);
            }
            self.samples.push(s.clone());
            self.last = Some(s);
        }
    }

    /// The last frame's ms (0 before one).
    pub fn last_frame(&self) -> f64 {
        self.frames.back().map_or(0.0, |f| f.1)
    }

    /// The worst frame that ended within `WORST_WINDOW_MS` of `now`.
    pub fn worst_recent(&self, now: f64) -> f64 {
        self.frames.iter().filter(|(end, _)| now - end <= WORST_WINDOW_MS).map(|f| f.1).fold(0.0, f64::max)
    }

    /// The overlay's line.
    pub fn overlay_text(&self, now: f64) -> String {
        let input = self.last.as_ref().map_or_else(|| "-".to_string(), |s| format!("{:.1} ms ({})", s.to_present, s.kind));
        format!("frame {:.1} ms  worst 1s {:.1} ms  input {input}", self.last_frame(), self.worst_recent(now))
    }

    /// Clears what a report reads (samples and frame times), so the next one covers only what follows.
    pub fn take(&mut self) -> (Vec<Sample>, Vec<f64>) {
        (std::mem::take(&mut self.samples), std::mem::take(&mut self.frame_ms))
    }

    /// ENGINE-60: the stalls since the last call (none of them is in `take`'s frame times).
    pub fn take_stalls(&mut self) -> Vec<Stall> {
        std::mem::take(&mut self.stalls)
    }
}

/// A report over `samples` and `frames` (ms): p50, p95 and worst of input-to-state, input-to-present and frame time,
/// every sample, and the flagged ones (input over `INPUT_FLAG_MS`, frame over `FRAME_FLAG_MS`). ENGINE-60: an input
/// shown by a stalled frame is listed but in no statistic.
pub fn report(samples: &[Sample], frames: &[f64]) -> Value {
    let pick = |f: fn(&Sample) -> f64| Stats::of(&samples.iter().filter(|s| !s.stall).map(f).collect::<Vec<_>>());
    let brief = |s: Stats| {
        let r = |x: f64| crate::perf::round_to(x, 3);
        json!({"count": s.count, "p50": r(s.p50), "p95": r(s.p95), "worst": r(s.max)})
    };
    let slow_frames = frames.iter().filter(|f| **f > FRAME_FLAG_MS).count();
    json!({
        "to_state": brief(pick(|s| s.to_state)),
        "to_present": brief(pick(|s| s.to_present)),
        "frame_time": brief(Stats::of(frames)),
        "inputs": samples.iter().map(Sample::to_json).collect::<Vec<_>>(),
        "flagged_inputs": samples.iter().filter(|s| s.flagged() && !s.stall).map(Sample::to_json).collect::<Vec<_>>(),
        "frames_over": slow_frames,
        "input_flag_ms": INPUT_FLAG_MS,
        "frame_flag_ms": FRAME_FLAG_MS,
    })
}
