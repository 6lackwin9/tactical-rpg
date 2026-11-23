//! ENGINE-49 (WALK-FEEL1): how walking feels, from what a screen that moves something reports each frame
//! (`Screen::motion`) and the inputs a `feel` replay sent. Pure functions: the replay (`shell::headless`, `feel` with
//! `motion`) collects a `Frame` after every frame and a `Mark` at every input, and `report` reads them:
//!
//! - first move: from a press, a pad press or a click to the first frame the mover is drawn somewhere else, and to its
//!   first new cell (a press that moves nothing, into a wall, counts under `still`);
//! - cadence: the time between steps (new cells) with no input between them, per stretch, and how uneven it is;
//! - overshoot: the steps taken after the last held direction was let go;
//! - a turn: a press while another direction is held, the time to the first step its way and the gap before it;
//! - a long frame (`spike`): when the next steps came;
//! - the camera: how far it is from where it wants to be (`target`), and its largest move in one frame (per frame of
//!   the replay's step, so a long frame is no jump), in tiles.
//!
//! Times are on the run's clock (the shot clock in a scenario run), in ms from the replay's start.

use serde_json::{json, Value};

/// One frame as the screen reports it (`Screen::motion`): the mover's cell, where it is drawn (in cells, between two
/// while it glides), the camera and where it wants to be (physical px), and a tile's physical px.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub t: f64,
    pub cell: (i64, i64),
    pub drawn: (f64, f64),
    pub cam: (f64, f64),
    pub target: (f64, f64),
    pub tile: f64,
}

impl Frame {
    /// A frame from a screen's `motion` report at `t` (None when it reports none).
    pub fn of(t: f64, v: &Value) -> Option<Frame> {
        let pair = |k: &str| -> Option<(f64, f64)> { Some((v[k][0].as_f64()?, v[k][1].as_f64()?)) };
        let cell = (v["cell"][0].as_i64()?, v["cell"][1].as_i64()?);
        Some(Frame { t, cell, drawn: pair("drawn")?, cam: pair("cam")?, target: pair("target")?, tile: v["tile"].as_f64().filter(|t| *t > 0.0)? })
    }
}

/// An input of the replay: when, what (`press`, `release`, `pad_press`, `pad_release`, `click`, `key`, `pad`, `spike`), the
/// direction it gives (a movement key), and whether a direction was held before it and is held after it.
#[derive(Debug, Clone, PartialEq)]
pub struct Mark {
    pub t: f64,
    pub kind: String,
    pub what: String,
    pub dir: Option<(i64, i64)>,
    pub held_before: bool,
    pub held_after: bool,
}

/// The steps the report lists (`path`: time and new cell), at most.
pub const PATH_MAX: usize = 400;

/// A step gap counts as a stutter past this many times the stretch's usual one (plus a frame).
pub const STUTTER_FACTOR: f64 = 1.5;

fn r2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// The times (and the step) of every new cell, in order: (t, from, to).
fn steps(frames: &[Frame]) -> Vec<(f64, (i64, i64), (i64, i64))> {
    frames.windows(2).filter(|w| w[0].cell != w[1].cell).map(|w| (w[1].t, w[0].cell, w[1].cell)).collect()
}

fn median(v: &[f64]) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.total_cmp(b));
    Some(s[s.len() / 2])
}

/// The report (see the module) over `frames` and `marks`, a frame lasting `frame_ms`.
pub fn report(frames: &[Frame], marks: &[Mark], frame_ms: f64) -> Value {
    let steps = steps(frames);
    let at_or_before = |t: f64| frames.iter().rev().find(|f| f.t <= t).or(frames.first());
    // the stretches between inputs: the gaps between their steps (the first step of a stretch counts from its input)
    let mut bounds: Vec<f64> = marks.iter().map(|m| m.t).collect();
    bounds.push(f64::INFINITY);
    let mut stretches: Vec<Vec<f64>> = Vec::new();
    for (k, w) in bounds.windows(2).enumerate() {
        // the stretch after a long frame is read on its own (`next_steps_ms`): its walk catches up on the beat
        if marks[k].kind == "spike" {
            continue;
        }
        let inside: Vec<f64> = steps.iter().filter(|s| s.0 > w[0] && s.0 <= w[1]).map(|s| s.0).collect();
        stretches.push(inside.windows(2).map(|p| p[1] - p[0]).collect());
    }
    let all_gaps: Vec<f64> = stretches.iter().flatten().copied().collect();
    let usual = median(&all_gaps);
    let mut inputs = Vec::new();
    let (mut first_worst, mut overshoot, mut still) = (0.0f64, 0u64, 0u64);
    for (i, m) in marks.iter().enumerate() {
        let end = marks.get(i + 1).map_or(f64::INFINITY, |n| n.t);
        let base = at_or_before(m.t);
        let mut row = json!({"t_ms": r2(m.t), "input": m.kind, "what": m.what});
        let moves = matches!(m.kind.as_str(), "press" | "pad_press" | "click" | "key" | "pad");
        if let (true, Some(b)) = (moves, base) {
            let moved = frames.iter().find(|f| f.t > m.t && f.t <= end && f.drawn != b.drawn).map(|f| f.t - m.t);
            let stepped = steps.iter().find(|s| s.0 > m.t && s.0 <= end).map(|s| s.0 - m.t);
            row["to_move_ms"] = json!(moved.map(r2));
            row["to_step_ms"] = json!(stepped.map(r2));
            // a press into a wall moves nothing: it is listed (`still`), not timed
            match moved {
                Some(ms) if !m.held_before || m.kind == "click" => first_worst = first_worst.max(ms),
                None => still += 1,
                _ => {}
            }
        }
        if let (Some(d), true, true) = (m.dir, m.held_before, matches!(m.kind.as_str(), "press" | "pad_press" | "key" | "pad")) {
            // a turn: the first step its way, and the gap before it
            let turned = steps.iter().find(|s| s.0 > m.t && s.0 <= end && (s.2 .0 - s.1 .0, s.2 .1 - s.1 .1) == d);
            if let Some(s) = turned {
                let before = steps.iter().rev().find(|p| p.0 < s.0).map(|p| p.0);
                let gap = before.map(|b| s.0 - b);
                row["turn_ms"] = json!(r2(s.0 - m.t));
                row["turn_gap_ms"] = json!(gap.map(r2));
                // a stutter: the walk was under way at the press (a step within a usual gap before it) and the turn's step
                // came later than a usual gap allows
                let walking = before.zip(usual).is_some_and(|(b, u)| m.t - b <= u + frame_ms);
                row["stutter"] = json!(walking && gap.zip(usual).is_some_and(|(g, u)| g > u * STUTTER_FACTOR + frame_ms));
            } else {
                row["turn_ms"] = Value::Null;
            }
        }
        if m.kind == "spike" {
            // a long frame: when the next steps came (a late one, then the walk back on its beat)
            let next: Vec<f64> = steps.iter().filter(|s| s.0 > m.t && s.0 <= end).take(3).map(|s| r2(s.0 - m.t)).collect();
            row["next_steps_ms"] = json!(next);
        }
        if matches!(m.kind.as_str(), "release" | "pad_release") && !m.held_after {
            // a step that starts after the release (one under way at the release already counts as taken)
            let n = steps.iter().filter(|s| s.0 > m.t + frame_ms * 0.5 && s.0 <= end).count() as u64;
            row["after_release_steps"] = json!(n);
            overshoot += n;
        }
        inputs.push(row);
    }
    let spread = stretches.iter().filter(|g| g.len() >= 2).map(|g| g.iter().copied().fold(f64::MIN, f64::max) - g.iter().copied().fold(f64::MAX, f64::min)).fold(0.0, f64::max);
    let lag = frames.iter().map(|f| ((f.cam.0 - f.target.0).powi(2) + (f.cam.1 - f.target.1).powi(2)).sqrt() / f.tile).fold(0.0, f64::max);
    // the camera's move in a frame, per frame of `frame_ms` (a long frame moves it further, at the same speed)
    let jump = frames.windows(2).map(|w| ((w[1].cam.0 - w[0].cam.0).powi(2) + (w[1].cam.1 - w[0].cam.1).powi(2)).sqrt() / w[1].tile * (frame_ms / (w[1].t - w[0].t).max(frame_ms))).fold(0.0, f64::max);
    json!({
        "frames": frames.len(),
        "steps": steps.len(),
        "frame_ms": r2(frame_ms),
        "first_move_ms": r2(first_worst),
        "still": still,
        "cadence_ms": {"count": all_gaps.len(), "min": all_gaps.iter().copied().reduce(f64::min).map(r2), "median": usual.map(r2), "max": all_gaps.iter().copied().reduce(f64::max).map(r2)},
        "cadence_spread_ms": r2(spread),
        "overshoot_steps": overshoot,
        "stutters": inputs.iter().filter(|r| r["stutter"] == json!(true)).count(),
        "camera_lag_tiles": r2(lag),
        "camera_jump_tiles": r2(jump),
        "inputs": inputs,
        "path": steps.iter().take(PATH_MAX).map(|s| json!([r2(s.0), s.2 .0, s.2 .1])).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A walk right with frames every `f` ms: a step at each frame index of `steps_at`, gliding over `glide` ms (drawn
    /// moving from the step's own frame).
    fn walk(f: f64, frames: usize, steps_at: &[usize], glide: f64) -> Vec<Frame> {
        (0..frames)
            .map(|k| {
                let done: Vec<&usize> = steps_at.iter().filter(|s| **s <= k).collect();
                let n = done.len() as i64;
                let x = match done.last() {
                    Some(s) => (n - 1) as f64 + (((k - **s) as f64 + 1.0) * f / glide).min(1.0),
                    None => 0.0,
                };
                Frame { t: k as f64 * f, cell: (n, 0), drawn: (x, 0.0), cam: (x * 32.0, 0.0), target: (x * 32.0, 0.0), tile: 32.0 }
            })
            .collect()
    }

    fn mark(t: f64, kind: &str, dir: Option<(i64, i64)>, before: bool, after: bool) -> Mark {
        Mark { t, kind: kind.into(), what: kind.into(), dir, held_before: before, held_after: after }
    }

    #[test]
    fn an_even_walk_moves_on_the_next_frame_and_stops_at_the_release() {
        let f = 1000.0 / 60.0;
        let frames = walk(f, 60, &[1, 13, 25], 200.0);
        let marks = [mark(0.0, "press", Some((1, 0)), false, true), mark(500.0, "release", Some((1, 0)), true, false)];
        let r = report(&frames, &marks, f);
        assert_eq!(r["steps"], 3);
        assert!(r["first_move_ms"].as_f64().unwrap() <= f + 0.01, "{r}");
        assert_eq!(r["overshoot_steps"], 0);
        assert!(r["cadence_spread_ms"].as_f64().unwrap() <= f, "{r}");
        assert_eq!(r["camera_lag_tiles"], 0.0);
    }

    #[test]
    fn a_step_after_the_release_and_an_uneven_gap_are_named() {
        let f = 1000.0 / 60.0;
        let frames = walk(f, 90, &[1, 12, 36, 54], 150.0);
        let marks = [mark(0.0, "press", Some((1, 0)), false, true), mark(800.0, "release", Some((1, 0)), true, false)];
        let r = report(&frames, &marks, f);
        assert_eq!(r["overshoot_steps"], 1, "{r}");
        assert!(r["cadence_spread_ms"].as_f64().unwrap() > 100.0, "{r}");
    }

    #[test]
    fn a_long_frame_is_read_apart_and_moves_the_camera_at_its_own_speed() {
        let f = 1000.0 / 60.0;
        // steps every 12 frames, then a 250 ms frame after frame 36 (the walk catches up on its beat)
        let mut frames = walk(f, 60, &[1, 13, 25, 37, 45, 57], 200.0);
        for fr in frames.iter_mut().skip(37) {
            fr.t += 250.0 - f;
        }
        let marks = [mark(0.0, "press", Some((1, 0)), false, true), mark(36.5 * f, "spike", None, true, true)];
        let r = report(&frames, &marks, f);
        assert!(r["cadence_spread_ms"].as_f64().unwrap() <= f, "the stretch before it is even: {r}");
        assert_eq!(r["inputs"][1]["next_steps_ms"].as_array().map(Vec::len), Some(3), "{r}");
        assert!(r["camera_jump_tiles"].as_f64().unwrap() < 0.5, "a long frame is no jump: {r}");
    }

    #[test]
    fn the_camera_lag_and_a_jump_count_in_tiles() {
        let mut frames = walk(10.0, 10, &[1], 50.0);
        frames[5].cam.0 += 64.0;
        let r = report(&frames, &[], 10.0);
        assert!(r["camera_lag_tiles"].as_f64().unwrap() >= 2.0 && r["camera_jump_tiles"].as_f64().unwrap() >= 2.0, "{r}");
    }
}
