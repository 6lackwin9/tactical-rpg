//! BOSSUI1 (PR7-54, `games/emberwake/docs/specs/SCREEN_CONTRACT_BATTLE.md` 9.1 and 9.2, take 32): the boss bar across
//! the board's top, drawn from the view-model's `BossBar` (the kit's `Session::phase_bar` and `intent::foe_card` over
//! the live plan, cut at a phase break by `intent::phase_cut`). Row 1: the portrait with the crown, the phase markers,
//! the unit's name and title (BU3b, PR7-59: "<name>, <form>" under a phase `form`; a phase's name shows only in the
//! phase banner) and the passive tags (hover: the description). Row 2: the phase's
//! HP span with the planned loss and "160/450". Row 3: poise as one bar with a tick per pip and "6/10", the per-pip
//! tag, icon-only status tabs, then the boss's own next blow. While planning, the two intent rows (the selected hero,
//! "Others") by block with totals; hovering them lists every member with a Total column and a "Party" line. Every impact
//! sits on fixed cells (HP right aligned, the poise glyph at a fixed x and its number, one cell per status chip, "+N").

use super::plates::{crown, icon, tab, Looks, SPLIT_SHIELD};
use crate::battle_view::{BossBar, BossRows, ImpactCell, ImpactRow};
use crate::portraits::Faces;
use crate::screens::Cx;
use crate::tokens::{boss_gold, frame_a, oxblood, planned};
use aegis_ui::canvas::{rgb, rgba, Rect, Rgba};
use aegis_ui::painter::{Painter, Typeface};
use aegis_ui::theme::th;
use aegis_ui::ui::{Tip, Ui, BODY, SMALL};
use emberwake_battle::session::PhaseMark;

/// Contract 9.1: the bar is 84 px tall (42 here), plus 58 px (29) of intent rows while planning.
pub(super) const BAR_H: i32 = 42;
pub(super) const ROWS_H: i32 = 29;
/// The board inset of the bar.
pub(super) const INSET: i32 = 3;
/// The portrait's side and the text column's start.
const FACE: i32 = 28;
const TEXT_X: i32 = 36;
/// Take 32's hidden cells: HP 36 px right aligned (18), the poise glyph at a fixed x then its number, one 17 px cell per
/// status chip (8.5, drawn 9 apart), then a "+N" cell.
pub(super) const HP_CELL: i32 = 18;
pub(super) const POISE_X: i32 = 20;
pub(super) const POISE_CELL: i32 = 15;
pub(super) const CHIP_CELL: i32 = 9;
pub(super) const MORE_CELL: i32 = 8;
/// Block cells hold one chip cell, the Totals three (contract 9.2), the hover Totals four (take 32).
pub(super) const BLOCK_CHIPS: usize = 1;
pub(super) const TOTAL_CHIPS: usize = 3;
pub(super) const HOVER_TOTAL_CHIPS: usize = 4;
/// The rows' name column.
const NAME_W: i32 = 36;
/// Row 3: the poise bar's width (80 px).
const POISE_BAR_W: i32 = 40;
/// The hover panel: its line pitch.
const LINE_H: i32 = 9;
/// The poise glyph and the broken-poise glyph: the kit's badge files (`AI-40`, `kits/rpg/assets/ui/badges/`); a missing
/// file falls back to the shape drawn in code (PR7-59).
const POISE_GLYPH: &str = "ui/badges/poise.png";
const BROKEN_POISE: &str = "ui/badges/broken_poise.png";

/// BU11, BU12 (PR7-59): the most the Stage board shifts down under the bar: the front face and its drop shadow under
/// the near edge, which leave the board's clip first (no row of cells leaves it).
pub(super) const MAX_LIFT: i32 = crate::stage::FACE_H + 8;

/// BU11, BU12 (PR7-59): how far the board shifts down under a boss bar so the bar covers no unit and no tile the plan
/// can target (in bounds and walkable, or holding a unit); the bar may lie over wall rows. Read against the grown bar
/// (with the intent rows), so the board keeps its place while a hero targets the boss and after; 0 in the Flat view
/// (its cells fill the panel) and at most `MAX_LIFT`.
pub(super) fn board_lift(w: &emberwake_battle::world::World, geo: &crate::stage::Geom) -> i32 {
    use emberwake_battle::grid::{V2, GRID_H, GRID_W};
    if geo.view != crate::stage::View::Stage {
        return 0;
    }
    let board = super::draw::BOARD;
    let bar = Rect::new(board.x + INSET, board.y + INSET, board.w - 2 * INSET, BAR_H + ROWS_H);
    let units: Vec<V2> = (0..w.units.len()).flat_map(|u| w.cells_of(u)).collect();
    let mut lift = 0;
    for y in 0..GRID_H {
        for x in 0..GRID_W {
            let c = V2::new(x, y);
            let r = geo.cell_rect(c);
            if super::skill_menu::overlaps(r, bar) && (w.grid.is_walkable(c) || units.contains(&c)) {
                lift = lift.max(bar.y + bar.h - r.y);
            }
        }
    }
    lift.clamp(0, MAX_LIFT)
}

/// The bar's height for `b` (contract 9.1: 84 px, 142 px with the intent rows).
pub(super) fn bar_h(b: &BossBar) -> i32 {
    BAR_H + if b.rows.is_some() { ROWS_H } else { 0 }
}

/// A cell's width with `chips` chip cells.
pub(super) fn cell_w(chips: usize) -> i32 {
    HP_CELL + 2 + POISE_CELL + chips as i32 * CHIP_CELL + MORE_CELL
}

/// The columns of the rows in a bar at `r`: the block columns' x and width, and the Totals column's x.
pub(super) fn columns(r: Rect, blocks: i32) -> (i32, i32, i32) {
    let x0 = r.x + 4 + NAME_W;
    let total_w = cell_w(TOTAL_CHIPS) + 2;
    let bw = ((r.w - 8 - NAME_W - total_w) / blocks.max(1)).max(cell_w(BLOCK_CHIPS));
    (x0, bw, x0 + bw * blocks.max(1))
}

/// A diamond marker centred at (`cx`, `cy`).
fn diamond(p: &mut Painter, cx: i32, cy: i32, half: i32, fill: Option<Rgba>, edge: Rgba) {
    if let Some(f) = fill {
        for k in -half..=half {
            let w = half - k.abs();
            p.fill(Rect::new(cx - w, cy + k, 2 * w + 1, 1), f);
        }
    }
    let (x, y, h) = (cx as f32 + 0.5, cy as f32 + 0.5, half as f32);
    for (a, b, c, d) in [(x - h, y, x, y - h), (x, y - h, x + h, y), (x + h, y, x, y + h), (x, y + h, x - h, y)] {
        p.line(a, b, c, d, edge, 1.0);
    }
}

/// The poise glyph in `r` (a small shield): the badge, or a drawn one.
fn poise_glyph(ui: &mut Ui<super::Action>, cx: &mut Cx, r: Rect, col: Rgba) {
    if !icon(ui, cx, POISE_GLYPH, r, 255) {
        let (x, y, w, h) = (r.x as f32, r.y as f32, r.w as f32, r.h as f32);
        ui.p.line(x, y, x + w, y, col, 1.0);
        ui.p.line(x, y, x, y + h * 0.5, col, 1.0);
        ui.p.line(x + w, y, x + w, y + h * 0.5, col, 1.0);
        ui.p.line(x, y + h * 0.5, x + w * 0.5, y + h, col, 1.0);
        ui.p.line(x + w, y + h * 0.5, x + w * 0.5, y + h, col, 1.0);
    }
}

/// The broken-poise glyph (BREAK) in `r`: the badge, the split shield, or a drawn one.
fn break_glyph(ui: &mut Ui<super::Action>, cx: &mut Cx, r: Rect) {
    if !icon(ui, cx, BROKEN_POISE, r, 255) && !icon(ui, cx, SPLIT_SHIELD, r, 255) {
        ui.p.panel(r, rgba(0, 0, 0, 0), planned(), 1.0);
        ui.p.line(r.x as f32 + r.w as f32 / 2.0, r.y as f32, r.x as f32 + r.w as f32 / 2.0, (r.y + r.h) as f32, planned(), 1.0);
    }
}

/// One impact cell at (`x`, `y`) with `chips` chip cells (take 32's hidden cells); `struck` greys it and strikes the
/// numbers.
#[allow(clippy::too_many_arguments)]
fn cell(ui: &mut Ui<super::Action>, cx: &mut Cx, id: &str, c: &ImpactCell, x: i32, y: i32, chips: usize, struck: bool) {
    let dim = struck;
    let hp_col = if dim { th().dim } else { th().foe };
    let hp = c.hp_text();
    let hp_col = if hp == "-" { th().dim } else { hp_col };
    ui.p.text_right((x + HP_CELL) as f32, y as f32, &hp, SMALL, hp_col);
    let hw = ui.p.text_w(&hp, SMALL).ceil() as i32;
    if dim && hp != "-" {
        ui.p.fill(Rect::new(x + HP_CELL - hw, y + 4, hw, 1), th().dim);
    }
    ui.push(format!("{id}.hp"), "label", hp.clone(), Rect::new(x + HP_CELL - hw, y, hw.max(1), 8), true, false, None, Vec::new());
    if c.pips > 0 {
        let px = x + POISE_X;
        poise_glyph(ui, cx, Rect::new(px, y + 1, 5, 6), if dim { th().dim } else { super::POISE });
        let n = c.pips.to_string();
        ui.p.text((px + 6) as f32, y as f32, &n, SMALL, if dim { th().dim } else { super::POISE });
        ui.push(format!("{id}.poise"), "label", n, Rect::new(px, y, POISE_CELL - 5, 8), true, false, None, Vec::new());
    }
    let cx0 = x + HP_CELL + 2 + POISE_CELL;
    for (k, st) in c.statuses.iter().take(chips).enumerate() {
        let r = Rect::new(cx0 + k as i32 * CHIP_CELL, y, 7, 7);
        if !icon(ui, cx, &emberwake_content::art::status_icon(st), r, if dim { 110 } else { 255 }) {
            ui.p.panel(r, rgba(0, 0, 0, 0), oxblood(), 1.0);
        }
        ui.p.panel(r, rgba(0, 0, 0, 0), oxblood(), 0.0);
    }
    if c.statuses.len() > chips {
        let more = format!("+{}", c.statuses.len() - chips);
        let mx = cx0 + chips as i32 * CHIP_CELL;
        ui.p.text(mx as f32, y as f32, &more, SMALL, th().text);
        ui.push(format!("{id}.more"), "label", more, Rect::new(mx, y, MORE_CELL, 8), true, false, None, Vec::new());
    }
}

/// One row (`row`) at `y`: the name (gold for the selected hero; `ring` rings its cells gold), a cell per block and
/// the total.
#[allow(clippy::too_many_arguments)]
fn impact_row(ui: &mut Ui<super::Action>, cx: &mut Cx, small: &mut Faces, looks: &Looks, rows: &BossRows, row: &ImpactRow, id: &str, r: Rect, y: i32, (hero, ring): (bool, bool), total_chips: usize) {
    let (x0, bw, tx) = columns(r, rows.blocks);
    let mut nx = r.x + 4;
    if let Some(l) = row.unit.and_then(|u| looks.get(u)) {
        small.draw(&mut ui.p, cx, &l.0, &l.1, super::TIMELINE_SURFACE, Rect::new(nx, y, 8, 8));
        nx += 10;
    }
    let name = ui.p.fit(&row.name, SMALL, (x0 - nx - 2) as f32);
    ui.p.text(nx as f32, y as f32, &name, SMALL, if hero { th().gold } else { th().text });
    ui.push(format!("{id}.name"), "label", row.name.clone(), Rect::new(nx, y, (x0 - nx - 2).max(1), 8), true, false, None, Vec::new());
    for (k, c) in row.cells.iter().enumerate() {
        let b = k as i32 + 1;
        let x = x0 + k as i32 * bw;
        if ring && c.hits > 0 && !rows.struck(b) {
            ui.p.panel(Rect::new(x + 1, y - 1, bw - 3, LINE_H), rgba(0x5a, 0x40, 0x10, 140), th().gold, 0.0);
        }
        cell(ui, cx, &format!("{id}.b{b}"), c, x + 2, y, BLOCK_CHIPS, rows.struck(b));
        if c.breaks && !rows.struck(b) {
            break_glyph(ui, cx, Rect::new(x + bw - 9, y, 7, 7));
        }
    }
    cell(ui, cx, &format!("{id}.total"), &row.total, tx + 2, y, total_chips, false);
}

/// The block headers at `y` ("Block N", "Block N: ends" ringed gold with the next phase's diamond, "Ph. 2 next round",
/// greyed blocks after the break) and the Totals header.
fn headers(ui: &mut Ui<super::Action>, rows: &BossRows, r: Rect, y: i32, total: bool) {
    let (x0, bw, tx) = columns(r, rows.blocks);
    for b in 1..=rows.blocks {
        let x = x0 + (b - 1) * bw;
        if b > 1 {
            ui.p.fill(Rect::new(x - 1, y, 1, r.y + r.h - y - 2), with_a(th().edge, 120));
        }
        if total {
            let text = rows.header(b);
            let ends = rows.ends == Some(b);
            let col = if ends || rows.ends.is_some_and(|e| b == e + 1) { th().gold } else { th().dim };
            if rows.struck(b) {
                ui.p.fill(Rect::new(x, y + 9, bw - 1, r.y + r.h - y - 11), rgba(60, 60, 70, 70));
            }
            ui.p.text_fit((x + 2) as f32, y as f32, &text, SMALL, (bw - 4) as f32, col);
            ui.push(format!("boss.head.{b}"), "label", text, Rect::new(x + 2, y, bw - 4, 8), true, false, None, Vec::new());
            if ends {
                ui.p.panel(Rect::new(x + 1, y + 9, bw - 3, LINE_H * 2 + 1), rgba(0, 0, 0, 0), th().gold, 0.0);
                diamond(&mut ui.p, x + bw - 6, y + 23, 3, Some(boss_gold()), boss_gold());
            }
        }
    }
    ui.p.fill(Rect::new(tx - 1, y, 1, r.y + r.h - y - 2), with_a(th().edge, 160));
    ui.p.text((tx + 2) as f32, y as f32, "Total", SMALL, th().dim);
}

fn with_a(c: Rgba, a: u8) -> Rgba {
    [c[0], c[1], c[2], a]
}

/// Contract 9.1, 9.2: the boss bar in `r` (`bar_h` tall). Returns the hover panel's rect when the rows are hovered (the
/// member list drawn under the bar).
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_bar(ui: &mut Ui<super::Action>, cx: &mut Cx, (faces, small): (&mut Faces, &mut Faces), looks: &Looks, b: &BossBar, r: Rect, tip: Tip) -> Option<Rect> {
    frame_a(&mut ui.p, r, rgba(12, 16, 26, 245));
    ui.push("boss.bar", "panel", b.name.clone(), r, true, false, None, tip);
    // row 1: the portrait with the crown, the markers, the title, the passive tags
    let pr = Rect::new(r.x + 4, r.y + 5, FACE, FACE);
    if let Some(l) = looks.get(b.unit) {
        faces.draw(&mut ui.p, cx, &l.0, &l.1, super::PANEL_SURFACE, pr);
        ui.push("boss.portrait", "portrait", l.0.name.clone(), pr, true, false, None, Vec::new());
    }
    ui.p.panel(pr, rgba(0, 0, 0, 0), boss_gold(), 0.0);
    crown(ui, cx, Rect::new(pr.x - 1, pr.y - 3, 8, 6));
    let tx = r.x + TEXT_X;
    let ty = r.y + 4;
    for (k, m) in b.marks.iter().enumerate() {
        let (cxm, cym) = (tx + 3 + k as i32 * 8, ty + 4);
        match m {
            PhaseMark::Current => {
                diamond(&mut ui.p, cxm, cym, 4, Some(rgb(0xff, 0xe0, 0x80)), th().text);
            }
            PhaseMark::Coming => diamond(&mut ui.p, cxm, cym, 3, Some(boss_gold()), boss_gold()),
            PhaseMark::Beaten => diamond(&mut ui.p, cxm, cym, 3, Some(rgb(70, 76, 90)), rgb(110, 116, 130)),
        }
        let state = match m {
            PhaseMark::Current => "current",
            PhaseMark::Coming => "to fight",
            PhaseMark::Beaten => "beaten",
        };
        ui.push(format!("boss.phase.{k}"), "badge", format!("Phase {}: {state}", k + 1), Rect::new(cxm - 4, cym - 4, 9, 9), true, false, None, vec![(format!("Phase {}", k + 1), th().gold), (state.to_string(), th().text)]);
    }
    let name_x = tx + b.marks.len() as i32 * 8 + 4;
    // the passive tags, right aligned
    let mut right = r.x + r.w - 5;
    let mut tags: Vec<(Rect, String, String)> = Vec::new();
    for (name, about) in b.passives.iter().rev() {
        let w = ui.p.text_w(name, SMALL).ceil() as i32 + 6;
        let tr = Rect::new(right - w, ty, w, 9);
        if tr.x < name_x + 60 {
            break;
        }
        right = tr.x - 3;
        tags.push((tr, name.clone(), about.clone()));
    }
    for (k, (tr, name, about)) in tags.iter().rev().enumerate() {
        ui.p.panel(*tr, rgba(16, 20, 30, 255), th().text, 1.5);
        ui.p.text(tr.x as f32 + 3.0, tr.y as f32 + 0.5, name, SMALL, th().text);
        let mut t: Tip = vec![(name.clone(), th().gold)];
        if !about.is_empty() {
            t.push((about.clone(), th().text));
        }
        ui.push(format!("boss.passive.{k}"), "badge", name.clone(), *tr, true, false, None, t);
    }
    let room = (right - name_x - 2) as f32;
    let title = ui.p.fit(&b.name, BODY, room);
    // PR7-63: the painter returns the end x, so the name's width is that less its start
    let used = (ui.p.text_face(Typeface::Display, name_x as f32, ty as f32 - 1.0, &title, BODY, boss_gold()) - name_x as f32).ceil() as i32;
    let name_tip: Tip = if title != b.name { vec![(b.name.clone(), boss_gold())] } else { Vec::new() };
    ui.push("boss.name", "label", b.name.clone(), Rect::new(name_x, ty - 1, used.max(1), 10), true, false, None, name_tip);
    if !b.title.is_empty() && (used as f32) + 20.0 < room {
        let st = ui.p.fit(&b.title, SMALL, room - used as f32 - 6.0);
        ui.p.text((name_x + used + 6) as f32, ty as f32 + 1.0, &st, SMALL, th().dim);
        let sw = ui.p.text_w(&st, SMALL).ceil() as i32 + 1;
        let title_tip: Tip = if st != b.title { vec![(b.title.clone(), th().dim)] } else { Vec::new() };
        ui.push("boss.title", "label", b.title.clone(), Rect::new(name_x + used + 6, ty + 1, sw, 8), true, false, None, title_tip);
    }
    // row 2: the phase's span with the planned loss, "160/450" and the loss under it
    let hy = r.y + 16;
    let num_w = 34;
    let hb = Rect::new(tx, hy, r.w - TEXT_X - num_w - 6, 5);
    ui.p.fill(hb, rgb(30, 20, 26));
    let span = b.span.max(1);
    let at = |v: i32| (hb.w * v.clamp(0, span)) / span;
    let kept = (b.span_left - b.planned).max(0);
    ui.p.fill(Rect::new(hb.x, hb.y, at(kept), hb.h), th().foe);
    ui.p.fill(Rect::new(hb.x + at(kept), hb.y, (at(b.span_left) - at(kept)).max(0), hb.h), rgb(120, 40, 40));
    let span_text = b.span_text();
    let nx = r.x + r.w - 5;
    ui.p.text_right(nx as f32, hy as f32 - 3.0, &span_text, SMALL, th().text);
    let sw = ui.p.text_w(&span_text, SMALL).ceil() as i32;
    ui.push("boss.hp", "label", span_text, Rect::new(nx - sw, hy - 3, sw + 1, 8), true, false, None, Vec::new());
    if b.planned > 0 {
        let loss = format!("-{}", crate::battle_view::impact_num(b.planned));
        ui.p.text_right(nx as f32, hy as f32 + 5.0, &loss, SMALL, th().foe);
        let lw = ui.p.text_w(&loss, SMALL).ceil() as i32;
        ui.push("boss.loss", "label", loss, Rect::new(nx - lw, hy + 5, lw + 1, 8), true, false, None, Vec::new());
    }
    // row 3: poise (one bar, a tick per pip, "6/10"), the per-pip tag, icon-only status tabs, the boss's own blow
    let py = r.y + 29;
    let mut x = tx;
    if let Some(po) = b.poise {
        poise_glyph(ui, cx, Rect::new(x, py, 6, 7), super::POISE);
        let pb = Rect::new(x + 9, py + 2, POISE_BAR_W, 3);
        ui.p.fill(pb, rgb(36, 44, 60));
        let of = po.of.clamp(1, crate::battle_view::BOSS_PIPS_MAX);
        let lit = po.lit.clamp(0, of);
        let px = |k: i32| pb.x + pb.w * k / of;
        let keep = if po.breaks { 0 } else { (lit - po.planned).max(0) };
        ui.p.fill(Rect::new(pb.x, pb.y, px(keep) - pb.x, pb.h), super::POISE);
        ui.p.fill(Rect::new(px(keep), pb.y, px(lit) - px(keep), pb.h), planned());
        for k in 1..of {
            ui.p.fill(Rect::new(px(k), pb.y, 1, pb.h), rgba(10, 14, 22, 200));
        }
        let t = po.text();
        ui.p.text((pb.x + pb.w + 3) as f32, py as f32, &t, SMALL, super::POISE);
        ui.push("boss.poise", "label", t.clone(), Rect::new(pb.x, py, pb.w + 22, 8), true, false, None, vec![("Poise".into(), super::POISE), (format!("{t} pips; each pip takes {} stagger", po.threshold), th().text)]);
        x = pb.x + pb.w + 24;
        let tag = format!("{}", po.threshold);
        let tw = ui.p.text_w(&tag, SMALL).ceil() as i32 + 11;
        let tr = Rect::new(x, py - 1, tw, 9);
        ui.p.panel(tr, rgb(30, 36, 50), th().edge, 2.0);
        poise_glyph(ui, cx, Rect::new(tr.x + 2, tr.y + 1, 5, 6), super::POISE);
        ui.p.text(tr.x as f32 + 9.0, tr.y as f32 + 0.5, &tag, SMALL, super::POISE);
        ui.push("boss.threshold", "badge", format!("T {}", po.threshold), tr, true, false, None, vec![("Per pip".into(), super::POISE), (format!("Each pip takes {} stagger", po.threshold), th().text)]);
        x = tr.x + tr.w + 3;
    }
    for (k, t) in b.tabs.iter().take(4).enumerate() {
        let r2 = Rect::new(x, py - 1, 9, 9);
        tab(ui, cx, t, r2);
        ui.push(format!("boss.status.{k}"), "status", t.line(), r2, true, false, None, vec![(t.name.clone(), if t.buff { th().gold } else { th().foe }), (t.line(), th().text)]);
        x += 10;
    }
    if b.tabs.len() > 4 {
        let more = format!("+{}", b.tabs.len() - 4);
        ui.p.text(x as f32, py as f32, &more, SMALL, th().text);
        let tip: Tip = b.tabs.iter().map(|t| (t.line(), if t.buff { th().gold } else { th().foe })).collect();
        ui.push("boss.status.more", "status", more, Rect::new(x, py - 1, 9, 9), true, false, None, tip);
        x += 10;
    }
    if let Some(bl) = &b.blow {
        x = x.max(tx + 150);
        ui.p.fill(Rect::new(x, py - 2, 1, 11), th().edge);
        x += 5;
        let ir = Rect::new(x, py - 1, 9, 9);
        let chain = crate::tokens::skill_icons(&cx.data, &bl.skill);
        if !chain.iter().any(|p| icon(ui, cx, p, ir, 255)) {
            ui.p.panel(ir, rgb(120, 40, 40), th().edge, 1.0);
        }
        x += 12;
        let nm = ui.p.fit(&bl.name, SMALL, 70.0);
        // PR7-63: the painter returns the end x (the blow's target, pips and loss were pushed off the bar)
        let w = (ui.p.text(x as f32, py as f32, &nm, SMALL, th().text) - x as f32).ceil() as i32;
        let blow_tip: Tip = if nm != bl.name { vec![(bl.name.clone(), th().text)] } else { Vec::new() };
        ui.push("boss.blow", "label", format!("{} block {}: -{}", bl.name, bl.impact, bl.damage), Rect::new(x, py, w.max(1), 8), true, false, None, blow_tip);
        x += w + 4;
        if let Some(l) = bl.target.and_then(|t| looks.get(t)) {
            small.draw(&mut ui.p, cx, &l.0, &l.1, super::TIMELINE_SURFACE, Rect::new(x, py - 1, 9, 9));
            x += 12;
        }
        for k in 1..=6 {
            ui.p.fill(Rect::new(x + (k - 1) * 6, py + 2, 5, 3), if k == bl.impact { th().foe } else { rgb(50, 56, 72) });
        }
        x += 40;
        if bl.damage > 0 {
            ui.p.text(x as f32, py as f32, &format!("-{}", crate::battle_view::impact_num(bl.damage)), SMALL, th().foe);
        }
    }
    let rows = b.rows.as_ref()?;
    // the intent rows (planning only): headers, the selected hero, "Others"
    let ry = r.y + BAR_H;
    ui.p.fill(Rect::new(r.x + 4, ry - 1, r.w - 8, 1), with_a(th().edge, 200));
    let rows_r = Rect::new(r.x, ry, r.w, ROWS_H);
    headers(ui, rows, Rect::new(r.x, r.y, r.w, r.h), ry + 1, true);
    if let Some(h) = &rows.hero {
        impact_row(ui, cx, small, looks, rows, h, "boss.hero", r, ry + 10, (true, true), TOTAL_CHIPS);
    }
    impact_row(ui, cx, small, looks, rows, &rows.others, "boss.others", r, ry + 19, (false, false), TOTAL_CHIPS);
    let open = ui.hover(rows_r);
    ui.push("boss.rows", "panel", "Party by member", rows_r, true, open, None, Vec::new());
    if !open {
        return None;
    }
    // the hover: every member's row with a Total column, then the Party line
    let hh = 12 + rows.members.len() as i32 * LINE_H + LINE_H + 4;
    let hr = Rect::new(r.x, r.y + r.h + 1, r.w, hh);
    frame_a(&mut ui.p, hr, rgba(12, 16, 26, 250));
    ui.push("boss.members", "panel", "Party by member", hr, true, false, None, Vec::new());
    ui.p.text_face(Typeface::Display, (hr.x + 5) as f32, (hr.y + 2) as f32, "Party by member", SMALL, boss_gold());
    let (_, _, ttx) = columns(r, rows.blocks);
    ui.p.text((ttx + 2) as f32, (hr.y + 2) as f32, "Total", SMALL, th().dim);
    let inner = Rect::new(hr.x, hr.y, hr.w, hr.h);
    for (k, m) in rows.members.iter().enumerate() {
        let hero = rows.hero.as_ref().is_some_and(|h| h.unit == m.unit);
        impact_row(ui, cx, small, looks, rows, m, &format!("boss.member.{k}"), inner, hr.y + 12 + k as i32 * LINE_H, (hero, false), HOVER_TOTAL_CHIPS);
    }
    let py2 = hr.y + 12 + rows.members.len() as i32 * LINE_H;
    ui.p.fill(Rect::new(hr.x + 4, py2, hr.w - 8, 1), with_a(th().edge, 200));
    impact_row(ui, cx, small, looks, rows, &rows.party, "boss.party", inner, py2 + 2, (false, false), HOVER_TOTAL_CHIPS);
    Some(hr)
}
