//! A multi-track timeline (TIMELINE1; Godot's TimelineModal, the "tactical timeline" suite, game-agnostic): a ruler of
//! blocks with the playhead and the selected block, then sections of tracks (one row per unit, say), each holding
//! segments that span blocks (an action from its start block to its end block). A track can be masked (its segments
//! hidden, a note shown instead: an enemy plan the player can't read). The screen builds a `Timeline` from its own
//! data each frame and draws it with `draw`; clicks come back as the screen's own actions (a block, a track, a
//! segment), hover cards are the segments' own. Rows past the area scroll (a kit scroll area).
//!
//! Parts and marks (RULESET1-E, engine-only): a segment can be drawn in parts (windup shaded, impact bright with its
//! landing edge, recovery hatched) or hatched whole; a track can link two of its segments (a chained follow-up) and
//! carry marks (a glyph in a colour with a hover card, inside one block's cell, also on a masked track). Marks take no
//! click, so clicks land where they did; labels and mask notes give way to them. All of it is empty by default.
//!
//! Ghost segments (INPUT1-U, engine-only): a segment can be a ghost, a planned, not chosen, span (what a planner filled
//! in for the player): its fill at `GHOST_ALPHA`, a dashed outline all round and its label in `DIM`; `draw_ghost` and
//! `ghost_fill` give a screen's own spans the same look. Off by default.

use crate::canvas::{rgb, rgba, Rect, Rgba};
use crate::ui::{tip, Tip, Ui, BODY, BORDER, CARD, DIM, GOLD, PANEL_HI, RADIUS, SMALL, TEXT};

/// Height of the block ruler, a section header and a track row (canvas pixels).
pub const RULER_H: i32 = 13;
pub const HEADER_H: i32 = 11;
pub const ROW_H: i32 = 15;
/// Width of the track labels' column at the least; it grows to the longest label (up to a quarter of the pane).
pub const LABEL_W: i32 = 74;

/// How a part of a segment is drawn over its fill.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartStyle {
    /// Shaded darker: the blocks before the segment takes effect.
    Windup,
    /// Brighter, with a light edge at its last block: where it takes effect.
    Impact,
    /// Hatched: the blocks after it, when the track is still busy.
    Recovery,
}

/// A run of a segment's blocks drawn in one style (first and last block, inclusive; cut to the segment's own span).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Part {
    pub start: i32,
    pub end: i32,
    pub style: PartStyle,
}

/// One span of blocks on a track (an action). `Default` is an empty, unstyled segment (no parts, not hatched).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Segment {
    /// Identifies the segment within its track (comes back with a click).
    pub key: String,
    /// First and last block, inclusive (1-based).
    pub start: i32,
    pub end: i32,
    pub label: String,
    pub fill: Rgba,
    /// Dashed outline (a planned move, say).
    pub dashed: bool,
    pub tip: Tip,
    /// Styled runs of blocks over the fill (none: one plain bar).
    pub parts: Vec<Part>,
    /// Hatched whole (a dimmed or cancelled span, say).
    pub hatched: bool,
    /// A ghost span (a planned, not chosen, span): its fill at `GHOST_ALPHA`, a dashed outline all round and the label
    /// in `DIM` (`draw_ghost`).
    pub ghost: bool,
}

/// A glyph in a colour at one block of a track, with its hover card; it takes no click.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Mark {
    /// Identifies the mark within its track (its widget id).
    pub key: String,
    pub block: i32,
    /// A short text, one or two characters ("!", "x", "+").
    pub glyph: String,
    pub colour: Rgba,
    pub tip: Tip,
}

/// A connector from the end of one segment to the start of another on the same track (a chained follow-up).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Link {
    /// The segments' keys.
    pub from: String,
    pub to: String,
    pub colour: Rgba,
    pub tip: Tip,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Track {
    pub key: String,
    pub label: String,
    pub colour: Rgba,
    pub segments: Vec<Segment>,
    /// Masked: the segments are hidden and this note is shown on the lane instead.
    pub masked: Option<String>,
    /// Marks in the blocks' cells; they show on a masked track too.
    pub marks: Vec<Mark>,
    /// Links between this track's segments (hidden while masked).
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub title: String,
    pub colour: Rgba,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Timeline {
    /// Blocks per round (the ruler's length).
    pub blocks: i32,
    /// The block being played (0: none, e.g. while planning).
    pub playhead: i32,
    pub selected_block: Option<i32>,
    pub selected_track: Option<String>,
    pub sections: Vec<Section>,
}

impl Timeline {
    /// The x range of block `b` (1-based) in a lane starting at `x` that is `w` wide.
    pub fn block_span(&self, x: i32, w: i32, b: i32) -> (i32, i32) {
        let n = self.blocks.max(1);
        let at = |i: i32| x + (w as f32 * i as f32 / n as f32).round() as i32;
        (at(b - 1), at(b))
    }

    /// The x range from block `a`'s left edge to block `b`'s right edge, both clamped to the ruler (`b` not before `a`).
    pub fn blocks_span(&self, x: i32, w: i32, a: i32, b: i32) -> (i32, i32) {
        let last = self.blocks.max(1);
        let a = a.clamp(1, last);
        (self.block_span(x, w, a).0, self.block_span(x, w, b.clamp(a, last)).1)
    }

    /// The bar of segment `s` on the track row `row` (its blocks, inset one pixel each side, two from the row's edges).
    pub fn bar(&self, s: &Segment, row: Rect, lane_x: i32, lane_w: i32) -> Rect {
        let (x0, x1) = self.blocks_span(lane_x, lane_w, s.start, s.end);
        Rect::new(x0 + 1, row.y + 2, x1 - x0 - 2, ROW_H - 4)
    }

    /// The rect of part `p` of segment `s` in its bar: the part's blocks cut to the segment's (None: no block left).
    pub fn part_rect(&self, s: &Segment, p: &Part, bar: Rect, lane_x: i32, lane_w: i32) -> Option<Rect> {
        let last = self.blocks.max(1);
        let ss = s.start.clamp(1, last);
        let (a, b) = (p.start.max(ss), p.end.min(s.end.clamp(ss, last)));
        if a > b {
            return None;
        }
        let (x0, x1) = self.blocks_span(lane_x, lane_w, a, b);
        let (x0, x1) = (x0.max(bar.x), x1.min(bar.x + bar.w));
        (x1 > x0).then(|| Rect::new(x0, bar.y, x1 - x0, bar.h))
    }

    /// The rect of each mark of `t` on the row (None: its block is off the ruler): inside its block's cell, marks of
    /// one block side by side, each as wide as its glyph and a margin (at least 7) but never wider than its share.
    pub fn mark_rects<A: Clone>(&self, ui: &mut Ui<A>, t: &Track, row: Rect, lane_x: i32, lane_w: i32) -> Vec<Option<Rect>> {
        (0..t.marks.len())
            .map(|i| {
                let b = t.marks[i].block;
                if b < 1 || b > self.blocks {
                    return None;
                }
                let n = t.marks.iter().filter(|m| m.block == b).count() as i32;
                let k = t.marks[..i].iter().filter(|m| m.block == b).count() as i32;
                let (x0, x1) = self.block_span(lane_x, lane_w, b);
                let (cx, cw) = (x0 + 1, x1 - x0 - 2);
                let (s0, s1) = (cx + cw * k / n, cx + cw * (k + 1) / n);
                let want = (ui.p.text_w(&t.marks[i].glyph, SMALL).ceil() as i32 + 4).max(7);
                let w = want.min(s1 - s0).max(1);
                Some(Rect::new(s0 + (s1 - s0 - w) / 2, row.y + 2, w, ROW_H - 4))
            })
            .collect()
    }

    /// The height the sections take (to size a pane or scroll).
    pub fn content_h(&self) -> i32 {
        self.sections.iter().map(|s| HEADER_H + s.tracks.len() as i32 * ROW_H + 3).sum()
    }

    /// Draws the timeline in `r`. Widget ids: `<id>.block.<b>`, `<id>.track.<track>`, `<id>.seg.<track>.<segment>`;
    /// `on_block(b)`, `on_track(track)` and `on_segment(track, segment)` make the actions they carry. Links and marks
    /// are info widgets (a hover card, no action): `<id>.link.<track>.<from>.<to>`, `<id>.mark.<track>.<mark>`.
    pub fn draw<A: Clone>(&self, ui: &mut Ui<A>, id: &str, r: Rect, on_block: impl Fn(i32) -> A, on_track: impl Fn(&str) -> A, on_segment: impl Fn(&str, &str) -> A) {
        let longest = self.sections.iter().flat_map(|s| &s.tracks).map(|t| ui.p.text_w(&t.label, BODY).ceil() as i32 + 9).max().unwrap_or(0);
        let label_w = longest.clamp(LABEL_W, (r.w / 4).max(LABEL_W));
        let lane_x = r.x + label_w;
        let lane_w = (r.w - label_w - 4).max(self.blocks.max(1));
        // The ruler: one tick per block, the playhead filled, the selected block outlined.
        ui.p.fill(Rect::new(r.x, r.y, r.w, RULER_H), rgba(16, 22, 36, 255));
        ui.p.text(r.x as f32 + 3.0, r.y as f32 + 2.5, "Block", SMALL, DIM);
        for b in 1..=self.blocks {
            let (x0, x1) = self.block_span(lane_x, lane_w, b);
            let cell = Rect::new(x0 + 1, r.y + 1, x1 - x0 - 2, RULER_H - 2);
            let (playing, selected) = (b == self.playhead, self.selected_block == Some(b));
            let hover = ui.hover(cell);
            ui.p.panel(cell, if playing { rgb(52, 96, 150) } else if hover { PANEL_HI } else { CARD }, if selected { GOLD } else if playing { rgb(128, 220, 255) } else { BORDER }, 2.0);
            ui.p.text_center((x0 + x1) as f32 / 2.0, r.y as f32 + 2.5, &format!("{}B{b}", if playing { "> " } else { "" }), SMALL, if playing || selected { GOLD } else { TEXT });
            ui.push(format!("{id}.block.{b}"), "button", format!("B{b}"), cell, true, selected, Some(on_block(b)), tip(&format!("Block {b}"), if playing { "The block being played." } else { "Select this block." }));
        }
        // The tracks scroll under the ruler.
        let view = Rect::new(r.x, r.y + RULER_H + 2, r.w, r.h - RULER_H - 2);
        let off = ui.scroll_begin(&format!("{id}.rows"), view, self.content_h());
        let mut y = view.y - off;
        for section in &self.sections {
            ui.p.text(r.x as f32 + 3.0, y as f32 + 1.5, &section.title, SMALL, section.colour);
            ui.p.fill(Rect::new(r.x + 3, y + HEADER_H - 1, r.w - 10, 1), rgba(section.colour[0], section.colour[1], section.colour[2], 90));
            y += HEADER_H;
            for t in &section.tracks {
                self.draw_track(ui, id, t, Rect::new(r.x, y, r.w, ROW_H), lane_x, lane_w, &on_track, &on_segment);
                y += ROW_H;
            }
            y += 3;
        }
        // The playhead's column over the lanes.
        if self.playhead > 0 {
            let (x0, x1) = self.block_span(lane_x, lane_w, self.playhead);
            ui.p.fill(Rect::new(x0, view.y, x1 - x0, view.h.min(self.content_h() - off)), rgba(128, 220, 255, 28));
        }
        ui.scroll_end();
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_track<A: Clone>(&self, ui: &mut Ui<A>, id: &str, t: &Track, row: Rect, lane_x: i32, lane_w: i32, on_track: &impl Fn(&str) -> A, on_segment: &impl Fn(&str, &str) -> A) {
        let selected = self.selected_track.as_deref() == Some(t.key.as_str());
        let label = Rect::new(row.x + 1, row.y + 1, lane_x - row.x - 3, ROW_H - 2);
        if selected || ui.hover(label) {
            ui.p.panel(label, if selected { rgba(86, 68, 22, 250) } else { PANEL_HI }, if selected { GOLD } else { BORDER }, 2.0);
        }
        ui.p.text_fit(label.x as f32 + 3.0, label.y as f32 + 2.0, &t.label, BODY, label.w as f32 - 6.0, t.colour);
        ui.push(format!("{id}.track.{}", t.key), "row", t.label.clone(), label, true, selected, Some(on_track(&t.key)), Vec::new());
        // The lane: one cell per block.
        for b in 1..=self.blocks {
            let (x0, x1) = self.block_span(lane_x, lane_w, b);
            ui.p.fill(Rect::new(x0 + 1, row.y + 2, x1 - x0 - 2, ROW_H - 4), rgba(24, 30, 44, 255));
        }
        let marks = self.mark_rects(ui, t, row, lane_x, lane_w);
        let kept: Vec<Rect> = marks.iter().flatten().copied().collect();
        if let Some(note) = &t.masked {
            let lane = Rect::new(lane_x + 1, row.y + 2, lane_w - 2, ROW_H - 4);
            for k in (0..lane.w).step_by(6) {
                ui.p.fill(Rect::new(lane.x + k, lane.y, 2, lane.h), rgba(60, 66, 84, 140));
            }
            let need = ui.p.text_w(note, SMALL).ceil() as i32;
            let (nx, nx1) = clear_of(lane.x + 4, lane.x + lane.w - 4, &kept, need);
            ui.p.text_fit(nx as f32, lane.y as f32 + 1.0, note, SMALL, (nx1 - nx) as f32, DIM);
            ui.push(format!("{id}.track.{}.masked", t.key), "row", note.clone(), lane, true, false, None, tip(&t.label, note));
            draw_marks(ui, id, t, &marks);
            return;
        }
        // Bars and their parts first, then the links over them, then the labels (clear of the marks) and the widgets.
        let bars: Vec<Rect> = t.segments.iter().map(|s| self.bar(s, row, lane_x, lane_w)).collect();
        for (s, &bar) in t.segments.iter().zip(&bars) {
            let hover = ui.hover(bar);
            let mut fill = if s.ghost { ghost_fill(s.fill) } else { s.fill };
            if hover {
                fill = [fill[0].saturating_add(30), fill[1].saturating_add(30), fill[2].saturating_add(30), fill[3]];
            }
            ui.p.panel(bar, fill, if hover { GOLD } else { rgba(0, 0, 0, 0) }, RADIUS);
            for p in &s.parts {
                if let Some(pr) = self.part_rect(s, p, bar, lane_x, lane_w) {
                    draw_part(ui, p.style, bar, pr);
                }
            }
            if s.hatched {
                hatch(ui, bar, HATCH);
            }
            if s.ghost {
                draw_ghost(ui, bar);
            } else if s.dashed {
                for x in (bar.x..bar.x + bar.w).step_by(4) {
                    ui.p.fill(Rect::new(x, bar.y, 2, 1), TEXT);
                    ui.p.fill(Rect::new(x, bar.y + bar.h - 1, 2, 1), TEXT);
                }
            }
        }
        let bar_of = |key: &str| t.segments.iter().position(|s| s.key == key).map(|i| bars[i]);
        let mut links = Vec::new();
        for l in &t.links {
            let (Some(a), Some(b)) = (bar_of(&l.from), bar_of(&l.to)) else { continue };
            // From the last pixels of `from` to the first of `to` (short of the label), along the bars' lower edge.
            let (x0, x1) = (a.x + a.w - 3, b.x + 3);
            let (x0, x1) = (x0.min(x1), x0.max(x1));
            let y = a.y + a.h - 2;
            ui.p.fill(Rect::new(x0, y, x1 - x0, 2), l.colour);
            ui.p.fill(Rect::new(x0, y - 3, 2, 5), l.colour);
            ui.p.fill(Rect::new(x1 - 2, y - 3, 2, 5), l.colour);
            links.push((l, Rect::new(x0, y - 3, x1 - x0, 5)));
        }
        for (s, &bar) in t.segments.iter().zip(&bars) {
            let need = ui.p.text_w(&s.label, SMALL).ceil() as i32;
            let (lx, lx1) = clear_of(bar.x + 3, bar.x + bar.w - 3, &kept, need);
            if (lx, lx1) == (bar.x + 3, bar.x + bar.w - 3) || lx1 - lx >= 6 {
                ui.p.text_fit(lx as f32, bar.y as f32 + 1.0, &s.label, SMALL, (lx1 - lx) as f32, if s.ghost { DIM } else { TEXT });
            }
            ui.push(format!("{id}.seg.{}.{}", t.key, s.key), "row", s.label.clone(), bar, true, false, Some(on_segment(&t.key, &s.key)), s.tip.clone());
        }
        for (l, r) in links {
            ui.push(format!("{id}.link.{}.{}.{}", t.key, l.from, l.to), "link", format!("{} > {}", l.from, l.to), r, true, false, None, l.tip.clone());
        }
        draw_marks(ui, id, t, &marks);
    }
}

/// The parts' overlays: a windup darker, an impact brighter with a light edge at its end, a recovery hatched.
const WINDUP_SHADE: Rgba = rgba(0, 0, 0, 90);
const IMPACT_GLOW: Rgba = rgba(255, 255, 255, 70);
const IMPACT_EDGE: Rgba = rgba(240, 244, 255, 220);
const RECOVERY_SHADE: Rgba = rgba(0, 0, 0, 40);
const HATCH: Rgba = rgba(0, 0, 0, 120);
/// A mark's backing, under its coloured outline and glyph.
const MARK_BG: Rgba = rgba(10, 14, 24, 235);

/// Part `style` over `bar`, drawn only in the part's rect `pr` (the bar's rounded ends stay rounded, the cuts between
/// parts straight). Public so a screen's own strip of blocks draws its spans in the widget's look (RULESET1-U slice 5).
pub fn draw_part<A: Clone>(ui: &mut Ui<A>, style: PartStyle, bar: Rect, pr: Rect) {
    ui.push_clip(pr);
    match style {
        PartStyle::Windup => ui.p.panel(bar, WINDUP_SHADE, rgba(0, 0, 0, 0), RADIUS),
        PartStyle::Impact => {
            ui.p.panel(bar, IMPACT_GLOW, rgba(0, 0, 0, 0), RADIUS);
            ui.p.fill(Rect::new(pr.x + pr.w - 2, pr.y, 2, pr.h), IMPACT_EDGE);
        }
        PartStyle::Recovery => {
            ui.p.panel(bar, RECOVERY_SHADE, rgba(0, 0, 0, 0), RADIUS);
            hatch(ui, pr, HATCH);
        }
    }
    ui.pop_clip();
}

/// A ghost span's fill opacity (a planned, not chosen, span: INPUT1 GH8), of the segment's own alpha.
pub const GHOST_ALPHA: f32 = 0.45;
/// The ghost outline's dash: `GHOST_DASH` pixels drawn, then as many skipped.
pub const GHOST_DASH: i32 = 2;

/// A ghost span's fill: `fill` at `GHOST_ALPHA` of its alpha.
pub fn ghost_fill(fill: Rgba) -> Rgba {
    [fill[0], fill[1], fill[2], (fill[3] as f32 * GHOST_ALPHA).round() as u8]
}

/// The ghost look's dashed outline round `bar` (all four sides, in `TEXT`). Public so a screen's own strip of blocks
/// draws its ghost spans in the widget's look (with `ghost_fill` for the fill and `DIM` for the label).
pub fn draw_ghost<A: Clone>(ui: &mut Ui<A>, bar: Rect) {
    let step = (GHOST_DASH * 2) as usize;
    for x in (bar.x..bar.x + bar.w).step_by(step) {
        let w = GHOST_DASH.min(bar.x + bar.w - x);
        ui.p.fill(Rect::new(x, bar.y, w, 1), TEXT);
        ui.p.fill(Rect::new(x, bar.y + bar.h - 1, w, 1), TEXT);
    }
    for y in (bar.y..bar.y + bar.h).step_by(step) {
        let h = GHOST_DASH.min(bar.y + bar.h - y);
        ui.p.fill(Rect::new(bar.x, y, 1, h), TEXT);
        ui.p.fill(Rect::new(bar.x + bar.w - 1, y, 1, h), TEXT);
    }
}

/// Diagonal hatching over `r`, one line every 4 pixels.
fn hatch<A: Clone>(ui: &mut Ui<A>, r: Rect, col: Rgba) {
    ui.push_clip(r);
    for k in (-r.h..r.w).step_by(4) {
        ui.p.line((r.x + k) as f32, (r.y + r.h - 1) as f32, (r.x + k + r.h - 1) as f32, r.y as f32, col, 0.0);
    }
    ui.pop_clip();
}

/// The marks of `t` in their rects (from `Timeline::mark_rects`): a dark chip outlined in the mark's colour, the glyph
/// on it; info widgets (the hover card, no click).
fn draw_marks<A: Clone>(ui: &mut Ui<A>, id: &str, t: &Track, rects: &[Option<Rect>]) {
    for (m, r) in t.marks.iter().zip(rects) {
        let Some(r) = *r else { continue };
        draw_mark(ui, &format!("{id}.mark.{}.{}", t.key, m.key), m, r);
    }
}

/// One mark in `r` (a dark chip outlined in its colour, the glyph on it) with its info widget `wid` (hover card, no
/// click). Public so a screen's own strip of blocks marks a block in the widget's look (RULESET1-U slice 5).
pub fn draw_mark<A: Clone>(ui: &mut Ui<A>, wid: &str, m: &Mark, r: Rect) {
    ui.p.panel(r, MARK_BG, m.colour, 2.0);
    if ui.p.text_w(&m.glyph, SMALL) <= (r.w - 2) as f32 {
        ui.p.text_center(r.x as f32 + r.w as f32 / 2.0, r.y as f32 + 1.0, &m.glyph, SMALL, m.colour);
    } else {
        ui.p.text_fit(r.x as f32 + 1.0, r.y as f32 + 1.0, &m.glyph, SMALL, (r.w - 2) as f32, m.colour);
    }
    ui.push(wid, "mark", m.glyph.clone(), r, true, false, None, m.tip.clone());
}

/// The stretch of `[x0, x1]` a text `need` wide may take with `marks` kept clear (a pixel either side): the first gap
/// between the marks it fits in, else the widest (the first of equals); `(x0, x0)` when the marks fill it all.
pub fn clear_of(x0: i32, x1: i32, marks: &[Rect], need: i32) -> (i32, i32) {
    let mut sorted: Vec<&Rect> = marks.iter().filter(|m| m.x + m.w > x0 && m.x < x1).collect();
    sorted.sort_by_key(|m| m.x);
    let mut gaps = Vec::new();
    let mut at = x0;
    for m in sorted {
        if m.x - 1 > at {
            gaps.push((at, m.x - 1));
        }
        at = at.max(m.x + m.w + 1);
    }
    if x1 > at {
        gaps.push((at, x1));
    }
    let widest = gaps.iter().copied().fold(None, |best: Option<(i32, i32)>, g| if best.is_none_or(|b| g.1 - g.0 > b.1 - b.0) { Some(g) } else { best });
    gaps.iter().copied().find(|g| g.1 - g.0 >= need).or(widest).unwrap_or((x0, x0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_split_the_lane_evenly_and_the_height_counts_every_row() {
        let t = Timeline { blocks: 6, sections: vec![Section { title: "A".into(), colour: TEXT, tracks: vec![Track { key: "x".into(), label: "X".into(), colour: TEXT, ..Track::default() }; 3] }], ..Timeline::default() };
        assert_eq!(t.block_span(100, 600, 1), (100, 200));
        assert_eq!(t.block_span(100, 600, 6), (600, 700));
        assert_eq!(t.blocks_span(100, 600, 2, 4), (200, 500));
        assert_eq!(t.blocks_span(100, 600, 5, 99), (500, 700), "the end is clamped to the ruler");
        assert_eq!(t.content_h(), HEADER_H + 3 * ROW_H + 3);
    }

    #[test]
    fn a_part_is_cut_to_its_segment_and_text_steps_round_the_marks() {
        let t = Timeline { blocks: 6, ..Timeline::default() };
        let s = Segment { start: 2, end: 4, ..Segment::default() };
        let bar = t.bar(&s, Rect::new(0, 0, 700, ROW_H), 100, 600);
        assert_eq!((bar.x, bar.w), (201, 298), "B2-B4 inset a pixel each side");
        let part = |a, b| t.part_rect(&s, &Part { start: a, end: b, style: PartStyle::Windup }, bar, 100, 600);
        assert_eq!(part(1, 2).map(|r| (r.x, r.w)), Some((201, 99)), "B1 is not the segment's: only B2 is drawn");
        assert_eq!(part(4, 9).map(|r| (r.x, r.w)), Some((400, 99)), "B4 up to the bar's end");
        assert_eq!(part(5, 6), None, "past the segment: nothing");
        let m = |x, w| Rect::new(x, 0, w, 10);
        assert_eq!(clear_of(10, 100, &[], 500), (10, 100), "no marks: the whole stretch, fitting or not");
        assert_eq!(clear_of(10, 100, &[m(8, 7)], 0), (16, 100), "a mark over the start: after it");
        assert_eq!(clear_of(10, 100, &[m(16, 5), m(8, 7)], 0), (22, 100), "marks side by side");
        assert_eq!(clear_of(10, 100, &[m(50, 7), m(80, 7)], 30), (10, 49), "the first gap the text fits in");
        assert_eq!(clear_of(10, 100, &[m(20, 7)], 30), (28, 100), "past a mark when the text does not fit before it");
        assert_eq!(clear_of(10, 100, &[m(50, 7), m(80, 7)], 45), (10, 49), "fits nowhere: the widest gap");
        assert_eq!(clear_of(10, 100, &[m(5, 100)], 10), (10, 10), "covered: no room");
    }
}
