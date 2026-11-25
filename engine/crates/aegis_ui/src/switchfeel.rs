//! ENGINE-51 (SWITCH-FEEL1): how long a screen switch takes, from what a `feel` replay with `switch` records: for each
//! input, the time its handler took and every frame drawn after it (its wall time, the top screen and the stack's depth,
//! and the transition a drawn screen showed over itself, `Screen::transition`). Pure functions: `report` reads them.
//!
//! - first: from the input to the first frame of the new screen (another top screen, or the same one at another depth);
//! - settled: to the first frame of the last screen it came to (a switch can chain: a story card hands over to a
//!   stage, the stage to a fight) drawn in a frame's time (`SETTLED_FRAME_MS`): the loads are done. It is followed
//!   until the screens stay put `QUIET_FRAMES` after that;
//! - long frames: a frame (with the handler, for the first) over `LONG_FRAME_MS` freezes what was shown before it; it is
//!   covered when that was a transition (a fade, a wipe, an entry beat), else it counts against the switch.
//!
//! A time is what a window at the replay's frame step would take: each frame counts its wall time or the step,
//! whichever is longer (a frame waits for the next vsync), and the handler counts as it ran.

use serde_json::{json, Value};

/// A frame over this freezes the picture long enough to see (the row's bar).
pub const LONG_FRAME_MS: f64 = 100.0;
/// A frame of the new screen this quick has nothing left to load (`feel::FRAME_FLAG_MS`).
pub const SETTLED_FRAME_MS: f64 = crate::feel::FRAME_FLAG_MS;
/// How much a dev build's long frames and budgets stretch: it draws about twice as slow (the same as the `feel` step's
/// `FEEL_DEV_ALLOWANCE` in `aegis_cli`).
pub const DEV_ALLOWANCE: f64 = 2.5;
/// The most frames a switch is followed before it is reported as never settling.
pub const FRAMES_MAX: usize = 600;
/// A settled switch is followed this many frames more, in case another screen follows on its own (3 s at 60 a second).
pub const QUIET_FRAMES: usize = 180;

/// One frame after a switch's input.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub wall_ms: f64,
    pub top: String,
    pub depth: usize,
    /// A transition a drawn screen showed in this frame (`Screen::transition`).
    pub transition: Option<String>,
}

/// One input and what followed it.
#[derive(Debug, Clone, PartialEq)]
pub struct Switch {
    pub what: String,
    pub handler_ms: f64,
    /// The top screen and depth before the input, and the transition shown then.
    pub from: (String, usize),
    pub from_transition: Option<String>,
    pub frames: Vec<Frame>,
    /// The step's limit for `own_ms` (None: recorded only).
    pub budget_ms: Option<f64>,
    /// The screen it is meant to reach (the script item's `to`): it settles on that screen's first quick frame and is
    /// followed no further (a screen opening after it on its own, a next fight, is not part of it). None: the last.
    pub to: Option<String>,
    /// A frame of the new screen this quick counts as settled: `SETTLED_FRAME_MS`, times the dev build's allowance in a
    /// dev build (ENGINE-52: its frames take about twice as long, so few came under 33 ms).
    pub settled_frame_ms: f64,
}

impl Switch {
    /// The first frame showing another screen than before the input.
    pub fn first(&self) -> Option<usize> {
        self.frames.iter().position(|f| (f.top.as_str(), f.depth) != (self.from.0.as_str(), self.from.1))
    }

    /// The first frame of the last screen it came to drawn in a frame's time (`settled_frame_ms`).
    pub fn settled(&self) -> Option<usize> {
        self.first()?;
        if let Some(to) = &self.to {
            return self.frames.iter().position(|f| &f.top == to && f.wall_ms <= self.settled_frame_ms);
        }
        let at = |i: usize| (self.frames[i].top.as_str(), self.frames[i].depth);
        let last = (1..self.frames.len()).rev().find(|&i| at(i) != at(i - 1)).unwrap_or(0);
        (last..self.frames.len()).find(|&i| self.frames[i].wall_ms <= self.settled_frame_ms && at(i) != (self.from.0.as_str(), self.from.1))
    }

    /// Whether the replay can stop following it: settled and quiet `QUIET_FRAMES` since (settled on its `to`: at once),
    /// no other screen in `QUIET_FRAMES` with no `to` (an input that moves a story card on a line; one with a `to`, a
    /// click that walks to an exit first, waits for it), or `FRAMES_MAX` frames.
    pub fn done(&self) -> bool {
        let quiet = if self.to.is_some() { 0 } else { QUIET_FRAMES };
        let unmoved = self.to.is_none() && self.first().is_none() && self.frames.len() >= QUIET_FRAMES;
        self.frames.len() >= FRAMES_MAX || unmoved || self.settled().is_some_and(|i| self.frames.len() - 1 - i >= quiet)
    }

    /// The time to the end of frame `i`, as a window at `step_ms` would take it.
    fn time_to(&self, i: usize, step_ms: f64) -> f64 {
        self.handler_ms + self.frames[..=i].iter().map(|f| f.wall_ms.max(step_ms)).sum::<f64>()
    }
}

fn r2(x: f64) -> f64 {
    crate::perf::round_to(x, 2)
}

/// The report of `switches` at the replay's frame step: per switch `what`, `from`, `to`, `handler_ms`, `first_ms`,
/// `settled_ms` (null when it never switched or settled), `chain` (the screens it went through), `own_ms` (from the old screen's last frame to settled: a walk
/// to an exit before the switch left out), `frames` (to settled), `worst_frame_ms`, `long_frames`
/// (each `ms` and `covered_by`: the transition shown before it, or null), `transitions` (those seen), `budget_ms` and
/// `over` (`own_ms` past the budget); over all `switches`, `worst_settled_ms`, `uncovered` (long frames no transition covered; the step's budget
/// `uncovered_frames` limits them) and `misses` (a switch with a budget over it, or one that never settled). `long` is
/// `LONG_FRAME_MS` times `k`, the dev build's allowance, which stretches the budgets too.
pub fn report(switches: &[Switch], step_ms: f64, k: f64) -> Value {
    let long = LONG_FRAME_MS * k;
    let mut rows = Vec::new();
    let (mut worst, mut uncovered) = (0.0f64, 0usize);
    let mut misses: Vec<String> = Vec::new();
    for sw in switches {
        let first = sw.first();
        let settled = sw.settled();
        let upto = settled.unwrap_or(sw.frames.len().saturating_sub(1));
        let chain: Vec<Value> = {
            let mut c: Vec<Value> = Vec::new();
            for f in sw.frames.iter().take(upto + 1) {
                let here = json!([f.top, f.depth]);
                if c.last() != Some(&here) {
                    c.push(here);
                }
            }
            c
        };
        let mut longs = Vec::new();
        let mut worst_frame = 0.0f64;
        let mut seen: Vec<String> = Vec::new();
        for (i, f) in sw.frames.iter().enumerate().take(upto + 1) {
            let wall = if i == 0 { f.wall_ms + sw.handler_ms } else { f.wall_ms };
            worst_frame = worst_frame.max(wall);
            if let Some(t) = f.transition.as_ref().filter(|t| !seen.contains(t)) {
                seen.push(t.clone());
            }
            if wall > long {
                let before = if i == 0 { sw.from_transition.clone() } else { sw.frames[i - 1].transition.clone() };
                if before.is_none() {
                    uncovered += 1;
                }
                longs.push(json!({"ms": r2(wall), "covered_by": before}));
            }
        }
        let settled_ms = settled.map(|i| r2(sw.time_to(i, step_ms)));
        // the switch's own part: from the end of the old screen's last frame (a walk or a beat before it left out)
        let own_ms = match (first, settled) {
            (Some(f), Some(i)) if f > 0 => Some(r2(sw.time_to(i, step_ms) - sw.time_to(f - 1, step_ms))),
            (Some(_), Some(i)) => Some(r2(sw.time_to(i, step_ms))),
            _ => None,
        };
        if let Some(ms) = settled_ms {
            worst = worst.max(ms);
        }
        let budget = sw.budget_ms.map(|b| b * k);
        // a budget holds the switch's own part: a walk to the exit or the old screen's fade out is play, not loading
        // (a long frame in them still counts against `uncovered`)
        let over = match (own_ms, budget) {
            (Some(ms), Some(b)) => ms > b,
            (None, Some(_)) => true,
            _ => false,
        };
        if over {
            misses.push(match own_ms {
                Some(ms) => format!("{}: settled {ms} ms after the old screen's last frame, over {} ms", sw.what, r2(budget.unwrap_or(0.0))),
                None => format!("{}: {}", sw.what, if first.is_none() { "no other screen came up" } else { "the new screen never settled" }),
            });
        }
        let to = sw.frames.get(upto).map_or(Value::Null, |f| json!([f.top, f.depth]));
        rows.push(json!({
            "what": sw.what, "from": [sw.from.0, sw.from.1], "to": to, "handler_ms": r2(sw.handler_ms),
            "first_ms": first.map(|i| r2(sw.time_to(i, step_ms))), "settled_ms": settled_ms, "own_ms": own_ms, "frames": settled.map(|i| i + 1),
            "worst_frame_ms": r2(worst_frame), "chain": chain, "long_frames": longs, "transitions": seen, "budget_ms": budget.map(r2), "over": over,
        }));
    }
    json!({"switches": rows, "worst_settled_ms": r2(worst), "uncovered": uncovered, "long_frame_ms": r2(long), "misses": misses})
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(ms: f64, top: &str, depth: usize, transition: Option<&str>) -> Frame {
        Frame { wall_ms: ms, top: top.into(), depth, transition: transition.map(String::from) }
    }

    fn switch(handler: f64, frames: Vec<Frame>, budget: Option<f64>) -> Switch {
        Switch { what: "click go".into(), handler_ms: handler, from: ("a".into(), 1), from_transition: None, frames, budget_ms: budget, to: None, settled_frame_ms: SETTLED_FRAME_MS }
    }

    #[test]
    fn a_switch_settles_on_the_first_quick_frame_of_the_new_screen() {
        // the old screen one more frame, the new one slow to load, then quick
        let sw = switch(5.0, vec![frame(10.0, "a", 1, None), frame(60.0, "b", 1, None), frame(8.0, "b", 1, None), frame(8.0, "b", 1, None)], Some(150.0));
        assert_eq!((sw.first(), sw.settled()), (Some(1), Some(2)));
        let r = report(&[sw], 16.0, 1.0);
        let row = &r["switches"][0];
        // 5 + 16 + 60 (+ 16 for the settled frame)
        assert_eq!((row["first_ms"].as_f64(), row["settled_ms"].as_f64(), row["frames"].as_u64()), (Some(81.0), Some(97.0), Some(3)));
        assert_eq!((row["to"].clone(), row["over"].clone()), (json!(["b", 1]), json!(false)));
        assert_eq!(row["own_ms"].as_f64(), Some(76.0), "the old screen's frame left out");
        assert_eq!(r["misses"], json!([]));
    }

    #[test]
    fn a_chained_switch_settles_on_its_last_screen() {
        let mut frames = vec![frame(9.0, "card", 2, None), frame(9.0, "stage", 1, None), frame(150.0, "fight", 2, None), frame(9.0, "fight", 2, None)];
        let sw = switch(1.0, frames.clone(), None);
        assert_eq!((sw.first(), sw.settled()), (Some(0), Some(3)));
        assert!(!sw.done(), "followed on for a while");
        let still = switch(1.0, (0..QUIET_FRAMES).map(|_| frame(9.0, "a", 1, None)).collect(), None);
        assert!(still.first().is_none() && still.done(), "no other screen for as long: done");
        frames.extend((0..QUIET_FRAMES).map(|_| frame(9.0, "fight", 2, None)));
        assert!(switch(1.0, frames, None).done());
        let r = report(&[sw], 16.0, 1.0);
        assert_eq!(r["switches"][0]["chain"], json!([["card", 2], ["stage", 1], ["fight", 2]]));
        assert_eq!(r["uncovered"], json!(1), "the fight's load froze the stage");
    }

    #[test]
    fn a_popup_is_a_switch_by_depth_and_a_budget_or_no_switch_misses() {
        let sw = switch(1.0, vec![frame(9.0, "a", 2, None)], Some(5.0));
        let r = report(&[sw], 16.0, 1.0);
        assert_eq!(r["switches"][0]["settled_ms"].as_f64(), Some(17.0));
        assert_eq!(r["misses"], json!(["click go: settled 17 ms after the old screen's last frame, over 5 ms"]));
        // stretched by k (a dev build), the same switch is within
        assert_eq!(report(&[switch(1.0, vec![frame(9.0, "a", 2, None)], Some(10.0))], 16.0, 2.5)["misses"], json!([]));
        let none = report(&[switch(1.0, vec![frame(9.0, "a", 1, None)], Some(100.0))], 16.0, 1.0);
        assert_eq!(none["misses"], json!(["click go: no other screen came up"]));
        assert_eq!(none["switches"][0]["settled_ms"], Value::Null);
    }

    #[test]
    fn a_budget_holds_the_switch_after_the_old_screens_last_frame() {
        // a walk to the exit (the old screen 20 frames) then a 60 ms load: 397 ms from the input, its own 76 ms
        let mut frames: Vec<Frame> = (0..20).map(|_| frame(8.0, "a", 1, None)).collect();
        frames.extend([frame(60.0, "fight", 2, None), frame(8.0, "fight", 2, None)]);
        let r = report(&[switch(1.0, frames, Some(150.0))], 16.0, 1.0);
        let row = &r["switches"][0];
        assert_eq!((row["settled_ms"].as_f64(), row["own_ms"].as_f64()), (Some(397.0), Some(76.0)));
        assert_eq!((row["over"].clone(), r["misses"].clone()), (json!(false), json!([])));
    }

    #[test]
    fn a_switch_with_a_to_screen_settles_there_and_is_followed_no_further() {
        // results fade out, the map loads, then a next fight opens on its own
        let frames = vec![frame(9.0, "results", 3, Some("fade_out")), frame(50.0, "map", 1, None), frame(9.0, "map", 1, None), frame(9.0, "map", 1, None), frame(80.0, "fight", 2, None), frame(9.0, "fight", 2, None)];
        let mut sw = Switch { from: ("results".into(), 3), to: Some("map".into()), ..switch(1.0, frames.clone(), Some(150.0)) };
        assert_eq!(sw.settled(), Some(2));
        assert!(sw.done(), "nothing to wait for once on the map");
        let row = &report(std::slice::from_ref(&sw), 16.0, 1.0)["switches"][0];
        assert_eq!((row["to"].clone(), row["chain"].clone()), (json!(["map", 1]), json!([["results", 3], ["map", 1]])));
        sw.to = None;
        assert_eq!(sw.settled(), Some(5), "without it the fight is where it ends");
    }

    #[test]
    fn a_dev_build_settles_on_its_stretched_frame() {
        let frames = vec![frame(100.0, "b", 1, None), frame(45.0, "b", 1, None), frame(45.0, "b", 1, None)];
        assert_eq!(switch(1.0, frames.clone(), None).settled(), None, "no frame under 33 ms");
        let dev = Switch { settled_frame_ms: SETTLED_FRAME_MS * DEV_ALLOWANCE, ..switch(1.0, frames, None) };
        assert_eq!(dev.settled(), Some(1));
    }

    #[test]
    fn a_long_frame_is_covered_only_when_a_transition_showed_before_it() {
        // the handler and the first frame freeze the old screen: 150 ms, nothing covering it
        let bare = report(&[switch(90.0, vec![frame(60.0, "b", 1, None), frame(5.0, "b", 1, None)], None)], 16.0, 1.0);
        assert_eq!((bare["uncovered"].clone(), bare["misses"].clone()), (json!(1), json!([])));
        assert_eq!(bare["switches"][0]["long_frames"], json!([{"ms": 150.0, "covered_by": null}]));
        // a fade shown first: the long frame after it is under the fade
        let faded = report(&[switch(1.0, vec![frame(9.0, "a", 1, Some("fade")), frame(300.0, "b", 1, Some("intro")), frame(5.0, "b", 1, None)], None)], 16.0, 1.0);
        assert_eq!(faded["uncovered"], json!(0));
        assert_eq!(faded["switches"][0]["long_frames"], json!([{"ms": 300.0, "covered_by": "fade"}]));
        assert_eq!(faded["switches"][0]["transitions"], json!(["fade", "intro"]));
    }
}
