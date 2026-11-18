//! ENGINE-2 (PERF-STEP1, `docs/specs/AGENTROLES1.md` section 5): frame-time numbers. `Stats` sums up a run of frame
//! times (p10, p50, p95, max, mean, in ms); `calib` times a fixed CPU workload, so a reader can tell a slow frame from a
//! loaded machine; `bench` times the painter's fill, panel and text on a canvas of a given size (`ui_bench`,
//! `tools/fc.sh bench`). The no-window shell answers the agent link's `perf` request with these.

use crate::canvas::{rgb, Canvas, Rect};
use crate::painter::{Fonts, Painter, View};
use aegis_core::clock::Instant;
use serde_json::{json, Value};
use std::hint::black_box;

/// A run of frame times summed up, in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stats {
    pub count: usize,
    pub p10: f64,
    pub p50: f64,
    pub p95: f64,
    pub max: f64,
    pub mean: f64,
}

impl Stats {
    /// The numbers of `ms` (any order); all 0 for none. A percentile is the nearest rank: the sample at or above that
    /// share of the run.
    pub fn of(ms: &[f64]) -> Self {
        if ms.is_empty() {
            return Self { count: 0, p10: 0.0, p50: 0.0, p95: 0.0, max: 0.0, mean: 0.0 };
        }
        let mut s = ms.to_vec();
        s.sort_by(f64::total_cmp);
        let rank = |p: f64| s[((p * s.len() as f64).ceil() as usize).clamp(1, s.len()) - 1];
        Self { count: s.len(), p10: rank(0.10), p50: rank(0.50), p95: rank(0.95), max: s[s.len() - 1], mean: s.iter().sum::<f64>() / s.len() as f64 }
    }

    /// The numbers rounded to 0.001 ms, for a reply or a file.
    pub fn to_json(&self) -> Value {
        let r = |x: f64| (x * 1000.0).round() / 1000.0;
        json!({"count": self.count, "p10": r(self.p10), "p50": r(self.p50), "p95": r(self.p95), "max": r(self.max), "mean": r(self.mean)})
    }
}

/// Milliseconds since `t`, as a fraction.
pub fn ms_since(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}

/// Rounds of the calibration workload: a few ms in a release build.
pub const CALIB_ROUNDS: u64 = 2_000_000;

/// The time of a fixed CPU workload (integer mixing, no memory traffic), in ms: the best of three, so one preemption
/// does not count. Two of them around a measurement that differ by much mean the machine's load changed during it.
pub fn calib() -> f64 {
    (0..3)
        .map(|_| {
            let t = Instant::now();
            let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
            for i in 0..black_box(CALIB_ROUNDS) {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                x = x.wrapping_add(i);
            }
            black_box(x);
            ms_since(t)
        })
        .fold(f64::INFINITY, f64::min)
}

/// How far apart two calibrations may be (as a share of the faster) before a run's numbers carry no verdict.
pub const CALIB_SPREAD_MAX: f64 = 0.20;

/// Whether calibrations `before` and `after` show a steady machine.
pub fn steady(before: f64, after: f64) -> bool {
    let low = before.min(after);
    low > 0.0 && (before.max(after) - low) / low <= CALIB_SPREAD_MAX
}

/// One benchmark: what it painted and its per-run times.
#[derive(Debug, Clone)]
pub struct Bench {
    pub name: &'static str,
    pub stats: Stats,
}

/// The painter's workloads on a `w` x `h` canvas, `runs` times each (fonts from `ttf`): a whole-canvas fill, 64
/// rounded panels, and 40 lines of body text. Each run is one whole workload, timed.
pub fn bench(ttf: &[u8], w: i32, h: i32, runs: usize) -> Vec<Bench> {
    let mut canvas = Canvas::new(w, h);
    let mut fonts = Fonts::new(ttf);
    let view = View::fit(w as u32, h as u32);
    let mut out = Vec::new();
    let mut time = |name: &'static str, canvas: &mut Canvas, fonts: &mut Fonts, f: &dyn Fn(&mut Painter)| {
        let mut ms = Vec::with_capacity(runs);
        for _ in 0..runs {
            let t = Instant::now();
            f(&mut Painter { c: canvas, v: view, fonts });
            ms.push(ms_since(t));
        }
        out.push(Bench { name, stats: Stats::of(&ms) });
    };
    time("fill", &mut canvas, &mut fonts, &|p| p.fill(Rect::new(0, 0, crate::ui::W, crate::ui::H), rgb(30, 40, 60)));
    time("panel", &mut canvas, &mut fonts, &|p| {
        for i in 0..64 {
            p.panel(Rect::new((i % 8) * 78 + 6, (i / 8) * 44 + 6, 72, 38), rgb(40, 50, 70), rgb(120, 130, 160), 3.0);
        }
    });
    time("text", &mut canvas, &mut fonts, &|p| {
        for i in 0..40 {
            p.text(8.0, 8.0 + i as f32 * 8.5, "The quick brown fox jumps over the lazy dog, 0123456789.", crate::ui::BODY, rgb(230, 230, 240));
        }
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_take_the_nearest_rank() {
        let ms: Vec<f64> = (1..=20).rev().map(f64::from).collect();
        let s = Stats::of(&ms);
        assert_eq!((s.count, s.p10, s.p50, s.p95, s.max), (20, 2.0, 10.0, 19.0, 20.0));
        assert_eq!(s.mean, 10.5);
        assert_eq!(Stats::of(&[7.0]).p10, 7.0, "one sample is every percentile");
        assert_eq!(Stats::of(&[]).count, 0);
        assert_eq!(Stats::of(&[1.23456]).to_json()["p50"], json!(1.235));
    }

    #[test]
    fn a_steady_machine_keeps_its_calibrations_within_a_fifth() {
        assert!(steady(10.0, 11.9) && steady(12.0, 10.0));
        assert!(!steady(10.0, 12.1) && !steady(0.0, 1.0));
        assert!(calib() > 0.0);
    }

    #[test]
    fn the_benchmarks_paint_and_time_each_workload() {
        let ttf = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap();
        let out = bench(&ttf, 640, 360, 2);
        let names: Vec<&str> = out.iter().map(|b| b.name).collect();
        assert_eq!(names, ["fill", "panel", "text"]);
        assert!(out.iter().all(|b| b.stats.count == 2));
    }
}
