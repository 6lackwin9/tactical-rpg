//! Compares two shots of one screen with a small tolerance (LAYOUT720-E2, docs/specs/LAYOUT720.md section 5; new, no
//! Godot file). Pure: pixels in, a report out, so a screen moved to relative layout can be checked against its picture
//! from before the move, where an edge may land up to one grid unit away.
//!
//! A pixel matches if some pixel of the other image within the radius is within `CHANNEL_TOLERANCE` on every channel.
//! The radius is `ceil(s)`, for `s` physical px per grid unit, and it is a square window: an edge moved one unit on both
//! axes still matches, which a round radius would refuse at the corner. Both images are read against each other, because
//! a line missing from one of them only shows from the side that has it.

use crate::canvas::{Image, Rect};
use anyhow::{bail, Result};
use std::fmt;

/// How far apart two channels of matching pixels may be.
pub const CHANNEL_TOLERANCE: u8 = 3;

/// The largest scale taken, in physical px per grid unit: a wider window would pass almost anything.
pub const MAX_SCALE: f32 = 16.0;

/// The result of `shot_diff`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShotDiff {
    /// Pixels with no match in the other image, both ways added up.
    pub unmatched: u64,
    /// The bounding box of those pixels, in physical px; None when there are none.
    pub bbox: Option<Rect>,
    /// The radius used, in physical px.
    pub radius: i32,
}

impl ShotDiff {
    /// True when every pixel of both images found a match.
    pub fn passed(&self) -> bool {
        self.unmatched == 0
    }
}

impl fmt::Display for ShotDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.bbox {
            Some(b) => write!(f, "{} pixels unmatched within {} px, bounding box x {} y {} w {} h {}", self.unmatched, self.radius, b.x, b.y, b.w, b.h),
            None => write!(f, "shots match within {} px", self.radius),
        }
    }
}

/// Compares `a` and `b` at `s` physical px per grid unit. Refuses, rather than crops or guesses, images of different
/// sizes, an image whose pixels do not fill its size, and a scale that is not above 0 and at most `MAX_SCALE`.
pub fn shot_diff(a: &Image, b: &Image, s: f32) -> Result<ShotDiff> {
    check(a, "first")?;
    check(b, "second")?;
    if (a.w, a.h) != (b.w, b.h) {
        bail!("shot_diff: shots differ in size, {} x {} and {} x {}", a.w, a.h, b.w, b.h);
    }
    if !(s > 0.0 && s <= MAX_SCALE) {
        bail!("shot_diff: scale {s} is not above 0 and at most {MAX_SCALE}");
    }
    let radius = s.ceil() as i32;
    let mut report = ShotDiff { unmatched: 0, bbox: None, radius };
    one_way(a, b, &mut report);
    one_way(b, a, &mut report);
    Ok(report)
}

fn check(i: &Image, which: &str) -> Result<()> {
    let expected = (i.w >= 0 && i.h >= 0).then(|| i.w as usize * i.h as usize * 4);
    if expected != Some(i.px.len()) {
        bail!("shot_diff: the {which} shot is {} x {} with {} bytes", i.w, i.h, i.px.len());
    }
    Ok(())
}

/// Adds every pixel of `from` with no match in `to`.
fn one_way(from: &Image, to: &Image, report: &mut ShotDiff) {
    let (w, h) = (from.w as usize, from.h as usize);
    let r = report.radius as usize;
    for y in 0..h {
        for x in 0..w {
            let own = &from.px[(y * w + x) * 4..][..4];
            let near = |xx: usize, yy: usize| close(own, &to.px[(yy * w + xx) * 4..][..4]);
            // The same place first: on a shot that matches, almost every pixel ends here.
            if near(x, y) || (y.saturating_sub(r)..(y + r + 1).min(h)).any(|yy| (x.saturating_sub(r)..(x + r + 1).min(w)).any(|xx| near(xx, yy))) {
                continue;
            }
            report.unmatched += 1;
            let hit = Rect::new(x as i32, y as i32, 1, 1);
            report.bbox = Some(report.bbox.map_or(hit, |b| union(b, hit)));
        }
    }
}

fn close(a: &[u8], b: &[u8]) -> bool {
    a.iter().zip(b).all(|(p, q)| p.abs_diff(*q) <= CHANNEL_TOLERANCE)
}

fn union(a: Rect, b: Rect) -> Rect {
    let (x0, y0) = (a.x.min(b.x), a.y.min(b.y));
    let (x1, y1) = ((a.x + a.w).max(b.x + b.w), (a.y + a.h).max(b.y + b.h));
    Rect::new(x0, y0, x1 - x0, y1 - y0)
}
