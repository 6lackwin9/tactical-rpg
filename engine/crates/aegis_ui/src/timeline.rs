//! A multi-track timeline (TIMELINE1; Godot's TimelineModal, the "tactical timeline" suite, game-agnostic): a ruler of
//! blocks with the playhead and the selected block, then sections of tracks (one row per unit, say), each holding
//! segments that span blocks (an action from its start block to its end block). A track can be masked (its segments
//! hidden, a note shown instead: an enemy plan the player can't read). The screen builds a `Timeline` from its own
//! data each frame and draws it with `draw`; clicks come back as the screen's own actions (a block, a track, a
//! segment), hover cards are the segments' own. Rows past the area scroll (a kit scroll area).

use crate::canvas::{rgb, rgba, Rect, Rgba};
use crate::ui::{tip, Tip, Ui, BODY, BORDER, CARD, DIM, GOLD, PANEL_HI, RADIUS, SMALL, TEXT};

/// Height of the block ruler, a section header and a track row (canvas pixels).
pub const RULER_H: i32 = 13;
pub const HEADER_H: i32 = 11;
pub const ROW_H: i32 = 15;
/// Width of the track labels' column.
pub const LABEL_W: i32 = 74;

/// One span of blocks on a track (an action).
#[derive(Debug, Clone, PartialEq)]
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub key: String,
    pub label: String,
    pub colour: Rgba,
    pub segments: Vec<Segment>,
    /// Masked: the segments are hidden and this note is shown on the lane instead.
    pub masked: Option<String>,
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

    /// The height the sections take (to size a pane or scroll).
    pub fn content_h(&self) -> i32 {
        self.sections.iter().map(|s| HEADER_H + s.tracks.len() as i32 * ROW_H + 3).sum()
    }

    /// Draws the timeline in `r`. Widget ids: `<id>.block.<b>`, `<id>.track.<track>`, `<id>.seg.<track>.<segment>`;
    /// `on_block(b)`, `on_track(track)` and `on_segment(track, segment)` make the actions they carry.
    pub fn draw<A: Clone>(&self, ui: &mut Ui<A>, id: &str, r: Rect, on_block: impl Fn(i32) -> A, on_track: impl Fn(&str) -> A, on_segment: impl Fn(&str, &str) -> A) {
        let lane_x = r.x + LABEL_W;
        let lane_w = (r.w - LABEL_W - 4).max(self.blocks.max(1));
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
        let label = Rect::new(row.x + 1, row.y + 1, LABEL_W - 3, ROW_H - 2);
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
        if let Some(note) = &t.masked {
            let lane = Rect::new(lane_x + 1, row.y + 2, lane_w - 2, ROW_H - 4);
            for k in (0..lane.w).step_by(6) {
                ui.p.fill(Rect::new(lane.x + k, lane.y, 2, lane.h), rgba(60, 66, 84, 140));
            }
            ui.p.text_fit(lane.x as f32 + 4.0, lane.y as f32 + 1.0, note, SMALL, lane.w as f32 - 8.0, DIM);
            ui.push(format!("{id}.track.{}.masked", t.key), "row", note.clone(), lane, true, false, None, tip(&t.label, note));
            return;
        }
        for s in &t.segments {
            let last = self.blocks.max(1);
            let start = s.start.clamp(1, last);
            let (x0, _) = self.block_span(lane_x, lane_w, start);
            let (_, x1) = self.block_span(lane_x, lane_w, s.end.clamp(start, last));
            let bar = Rect::new(x0 + 1, row.y + 2, x1 - x0 - 2, ROW_H - 4);
            let hover = ui.hover(bar);
            let mut fill = s.fill;
            if hover {
                fill = [fill[0].saturating_add(30), fill[1].saturating_add(30), fill[2].saturating_add(30), fill[3]];
            }
            ui.p.panel(bar, fill, if hover { GOLD } else { rgba(0, 0, 0, 0) }, RADIUS);
            if s.dashed {
                for x in (bar.x..bar.x + bar.w).step_by(4) {
                    ui.p.fill(Rect::new(x, bar.y, 2, 1), TEXT);
                    ui.p.fill(Rect::new(x, bar.y + bar.h - 1, 2, 1), TEXT);
                }
            }
            ui.p.text_fit(bar.x as f32 + 3.0, bar.y as f32 + 1.0, &s.label, SMALL, bar.w as f32 - 6.0, TEXT);
            ui.push(format!("{id}.seg.{}.{}", t.key, s.key), "row", s.label.clone(), bar, true, false, Some(on_segment(&t.key, &s.key)), s.tip.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_split_the_lane_evenly_and_the_height_counts_every_row() {
        let t = Timeline { blocks: 6, sections: vec![Section { title: "A".into(), colour: TEXT, tracks: vec![Track { key: "x".into(), label: "X".into(), colour: TEXT, segments: Vec::new(), masked: None }; 3] }], ..Timeline::default() };
        assert_eq!(t.block_span(100, 600, 1), (100, 200));
        assert_eq!(t.block_span(100, 600, 6), (600, 700));
        assert_eq!(t.content_h(), HEADER_H + 3 * ROW_H + 3);
    }
}
