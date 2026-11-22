//! BATTLE-UI2 (PR7-40, `games/emberwake/docs/specs/SCREEN_CONTRACT_BATTLE.md` 5): the docked foe card at the top right
//! of the board, drawn from the view-model's `CardView` (itself the kit's `intent::foe_card` over the live plan): the
//! header (portrait with a boss's crown, the full name and the character's title, the HP bar previewing the plan with
//! the selected hero's share bright and the others' dark, the poise pips with the pip threshold, the KO tag with the
//! crit star and the BREAK tag), then in the hero view a row per move of that hero on the foe (skill icon, a 6-block
//! ruler with the move's blocks lit, the damage, the pips each hit breaks or the split shield, status chips, the
//! after-KO tag) and in the party view a row per block (the block badge, the hitters' faces, the summed damage, the pips
//! per hit, the chips, "KO"), and the totals.

use super::plates::{crown, icon, Looks, CRIT_STAR, SPLIT_SHIELD};
use crate::battle_view::{CardRow, CardView, CARD_CHIPS};
use crate::portraits::Faces;
use crate::screens::Cx;
use crate::tokens::{boss_gold, frame_a, oxblood, planned};
use aegis_ui::canvas::{rgb, rgba, Rect};
use aegis_ui::painter::Typeface;
use aegis_ui::theme::th;
use aegis_ui::ui::{Ui, BODY, SMALL};

/// The card's width (316 px), its header's height and a row's.
pub(super) const CARD_W: i32 = 158;
const HEAD_H: i32 = 46;
const ROW_H: i32 = 12;
const FOOT_H: i32 = 22;
/// The poise area's 3 slots (the per-hit cap) and the mini ruler's blocks.
const POISE_SLOTS: i32 = 3;
const RULER_BLOCKS: i32 = 6;

/// The card's height for `c`.
pub(super) fn card_h(c: &CardView) -> i32 {
    HEAD_H + c.rows.len().max(1) as i32 * ROW_H + FOOT_H
}

/// The pips each hit of a row breaks in the poise area at `x` (amber pips, the split shield for the hit that empties
/// the bar, dimmed on an already broken bar).
fn poise_area(ui: &mut Ui<super::Action>, cx: &mut Cx, pips: &[(i32, bool, bool)], x: i32, y: i32) {
    let mut at = x;
    for (n, breaks, dim) in pips {
        let alpha = if *dim { 90 } else { 255 };
        if *breaks {
            let r = Rect::new(at, y, 7, 7);
            if !icon(ui, cx, SPLIT_SHIELD, r, alpha) {
                ui.p.panel(r, rgba(0, 0, 0, 0), planned(), 1.0);
                ui.p.line(r.x as f32 + 3.5, r.y as f32, r.x as f32 + 3.5, (r.y + r.h) as f32, planned(), 0.0);
            }
            at += 8;
            continue;
        }
        for _ in 0..(*n).clamp(0, POISE_SLOTS) {
            let [r0, g0, b0, _] = planned();
            ui.p.fill(Rect::new(at, y + 2, 3, 3), [r0, g0, b0, alpha]);
            at += 4;
        }
        at += 1;
    }
}

/// The status chips after the damage: at most `CARD_CHIPS`, then "+N".
fn chips(ui: &mut Ui<super::Action>, cx: &mut Cx, statuses: &[String], x: i32, y: i32) -> i32 {
    let mut at = x;
    for st in statuses.iter().take(CARD_CHIPS) {
        let r = Rect::new(at, y, 7, 7);
        if !icon(ui, cx, &emberwake_content::art::status_icon(st), r, 255) {
            ui.p.panel(r, rgba(0, 0, 0, 0), oxblood(), 1.0);
        }
        at += 8;
    }
    if statuses.len() > CARD_CHIPS {
        // PR7-63: the painter returns the end x
        at = ui.p.text(at as f32, y as f32, &format!("+{}", statuses.len() - CARD_CHIPS), SMALL, th().text).ceil() as i32 + 1;
    }
    at
}

/// A soft outline tag ("after KO") at `x`.
fn soft_tag(ui: &mut Ui<super::Action>, text: &str, x: i32, y: i32) {
    let w = ui.p.text_w(text, SMALL).ceil() as i32 + 4;
    ui.push(format!("card.after_ko.{x}.{y}"), "badge", text, Rect::new(x, y, w, 8), true, false, None, Vec::new());
    ui.p.panel(Rect::new(x, y, w, 8), rgba(0, 0, 0, 0), rgb(150, 130, 90), 1.0);
    let ty = tag_text_y(ui, y);
    ui.p.text(x as f32 + 2.0, ty, text, SMALL, rgb(190, 170, 120));
}

/// PR7-63: a tag's text top: half a pixel in, or centred on the 8 px tag when its line is taller (Extra large).
fn tag_text_y(ui: &mut Ui<super::Action>, y: i32) -> f32 {
    let lh = ui.p.line_h(SMALL);
    if lh > 8.0 { y as f32 + (8.0 - lh) / 2.0 } else { y as f32 + 0.5 }
}

/// A filled tag ("KO in block 5", "BREAK in block 4") at `x`; its width.
fn tag(ui: &mut Ui<super::Action>, text: &str, x: i32, y: i32, fill: aegis_ui::canvas::Rgba) -> i32 {
    let w = ui.p.text_w(text, SMALL).ceil() as i32 + 6;
    ui.p.panel(Rect::new(x, y, w, 9), fill, fill, 1.5);
    ui.p.text(x as f32 + 3.0, y as f32 + 1.0, text, SMALL, th().text);
    ui.push(format!("card.tag.{}", text.split(' ').next().unwrap_or("").to_lowercase()), "badge", text, Rect::new(x, y, w, 9), true, false, None, Vec::new());
    w
}

/// One row of the card at `y`.
#[allow(clippy::too_many_arguments)]
fn row(ui: &mut Ui<super::Action>, cx: &mut Cx, small: &mut Faces, looks: &Looks, c: &CardView, k: usize, r: &CardRow, x: i32, y: i32) {
    let mut at = x;
    if c.hero.is_some() {
        // the skill icon, then the 6-block ruler with the move's blocks lit
        let ir = Rect::new(at, y, 9, 9);
        let chain = crate::tokens::skill_icons(&cx.data, &r.skill);
        if !chain.iter().any(|p| icon(ui, cx, p, ir, 255)) {
            ui.p.panel(ir, rgb(40, 46, 60), th().edge, 1.0);
        }
        at += 11;
        for b in 1..=RULER_BLOCKS {
            let lit = b >= r.start && b <= r.impact;
            ui.p.fill(Rect::new(at + (b - 1) * 4, y + 4, 3, 2), if b == r.impact { th().foe } else if lit { planned() } else { rgb(60, 66, 84) });
        }
        at += RULER_BLOCKS * 4 + 3;
    } else {
        // the block badge, then the hitters' faces
        let br = Rect::new(at, y, 10, 9);
        ui.p.panel(br, rgb(30, 36, 50), th().edge, 1.0);
        ui.p.text_center(br.x as f32 + 5.0, y as f32 + 1.0, &r.start.to_string(), SMALL, th().gold);
        at += 12;
        for u in r.units.iter().take(4) {
            if let Some(l) = looks.get(*u) {
                small.draw(&mut ui.p, cx, &l.0, &l.1, super::TIMELINE_SURFACE, Rect::new(at, y, 9, 9));
            }
            at += 10;
        }
        at = at.max(x + 12 + 4 * 10);
    }
    // the damage: a blow shows the red heart and "-34" (the kit's heart badge is the heal's moss heart, contract 6)
    super::plates::heart_shape(&mut ui.p, Rect::new(at, y + 2, 6, 5), th().foe);
    let dmg = format!("-{}", r.damage);
    // STATUSVIS1 SV7 (PR7-57): the number after every modifier, tinted with an arrow when one moved it; its breakdown on
    // a frameless hover (pushed after the row, so it is the card under the mouse)
    let plain = if r.after_ko { th().dim } else { th().text };
    let col = super::statusvis::num_col(r.sign, plain);
    ui.p.text(at as f32 + 9.0, y as f32 + 1.0, &dmg, BODY, col);
    let dw = ui.p.text_w(&dmg, BODY);
    super::statusvis::arrow(&mut ui.p, at as f32 + 9.0 + dw + 1.0, y as f32 + 3.0, r.sign, col);
    line_box(ui, &format!("card.row.{k}.damage"), &dmg, Rect::new(at + 9, y, 24, ROW_H - 1), BODY);
    let num_r = Rect::new(at + 9, y, (dw.ceil() as i32 + 5).min(25), ROW_H - 1);
    at += 34;
    poise_area(ui, cx, &r.pips, at, y);
    at += POISE_SLOTS * 4 + 6;
    let mut after = chips(ui, cx, &r.statuses, at, y + 1);
    // SV9: the riposte chip, no number; its hover the likely return on each hitter
    let mut chip = None;
    if !r.riposte.is_empty() {
        chip = Some(after + 1);
        after += 2 + super::statusvis::riposte_chip_w(ui);
    }
    if r.after_ko {
        soft_tag(ui, "after KO", after + 1, y);
    } else if r.ko {
        tag(ui, "KO", after + 1, y, oxblood());
    }
    let tip = vec![(r.name.clone(), th().gold), (format!("Blocks {}-{}: -{} HP", r.start, r.impact, r.damage), th().text)];
    // SV13: a hovered number (or chip) gets no frame: the row's own hover outline stays off while the mouse is on it
    let chip_r = chip.map(|cx0| Rect::new(cx0, y, super::statusvis::riposte_chip_w(ui), 8));
    let on_number = (!r.parts.is_empty() && ui.hover(num_r)) || chip_r.is_some_and(|c| ui.hover(c));
    ui.push(format!("card.row.{k}"), if on_number { "label" } else { "row" }, format!("{} -{}", r.name, r.damage), Rect::new(x, y, CARD_W - 8, ROW_H - 1), true, false, None, tip);
    if !r.parts.is_empty() {
        ui.push(format!("card.row.{k}.number"), "label", dmg.clone(), num_r, true, false, None, super::statusvis::num_tip(&format!("{}: -{} HP", r.name, r.damage), &r.parts));
    }
    if let Some(cx0) = chip {
        let returns: Vec<(String, i32)> = r.riposte.iter().map(|(u, hp)| (looks.get(*u).map_or_else(|| format!("unit {u}"), |l| l.0.name.clone()), *hp)).collect();
        super::statusvis::riposte_chip(ui, &format!("card.row.{k}.riposte"), cx0, y, &returns);
    }
}

/// A footer text at (`x`, `y`) with its label widget.
fn foot(ui: &mut Ui<super::Action>, id: &str, text: &str, x: i32, y: i32, size: f32, col: aegis_ui::canvas::Rgba) {
    ui.p.text(x as f32, y as f32, text, size, col);
    line_box(ui, id, text, Rect::new(x, y, 80, 9), size);
}

/// A label widget over one of the card's text lines (the text audit then reads it against the card, not the board's
/// tiles under it).
fn line_box(ui: &mut Ui<super::Action>, id: &str, text: &str, r: Rect, size: f32) {
    let w = (ui.p.text_w(text, size).ceil() as i32 + 1).max(1);
    let h = (ui.p.line_h(size).ceil() as i32 + 1).max(r.h.min(4));
    ui.push(id, "label", text, Rect::new(r.x, r.y, w, h), true, false, None, Vec::new());
}

/// Contract 5: the docked card at `r` (its top right on the board's).
/// `id` and `label` are the card's widget and its text (the agent link's `intel.unit`: the foe's name, "Inspecting <name> (Esc clears)"
/// when pinned).
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_card(ui: &mut Ui<super::Action>, cx: &mut Cx, (faces, small): (&mut Faces, &mut Faces), looks: &Looks, c: &CardView, r: Rect, id: &str, label: &str, tip: aegis_ui::ui::Tip) {
    frame_a(&mut ui.p, r, rgba(16, 20, 30, 245));
    // the card's own widget first: its texts belong to it (the board's tiles under it come before)
    ui.push(id, "panel", label, r, true, false, None, tip);
    let (x, y) = (r.x + 5, r.y + 4);
    // header: the portrait (a boss's crown), the full name and the title
    let pr = Rect::new(x, y, super::plates::PORTRAIT, super::plates::PORTRAIT);
    if let Some(l) = looks.get(c.foe) {
        faces.draw(&mut ui.p, cx, &l.0, &l.1, super::PANEL_SURFACE, pr);
        ui.push("intel.portrait", "portrait", l.0.name.clone(), pr, true, false, None, Vec::new());
    }
    if c.boss {
        crown(ui, cx, Rect::new(pr.x - 1, pr.y - 2, 8, 6));
    }
    // OBJUI1 (PR7-31; MORALE1 3): the morale chip and the leader's pennant on the portrait's lower corners, their words
    // on hover
    if let Some(m) = &c.morale {
        super::objchips::morale_chip(ui, m, Rect::new(pr.x - 2, pr.y + pr.h - super::objchips::CHIP + 2, super::objchips::CHIP, super::objchips::CHIP), "card.morale");
    }
    if c.leader {
        super::objchips::leader_chip(ui, Rect::new(pr.x + pr.w - super::objchips::CHIP + 2, pr.y + pr.h - super::objchips::CHIP + 2, super::objchips::CHIP, super::objchips::CHIP), "card.leader");
    }
    let tx = pr.x + pr.w + 4;
    let room = (r.x + r.w - 5 - tx) as f32;
    let name = ui.p.fit(&c.name, BODY, room);
    ui.p.text_face(Typeface::Display, tx as f32, y as f32, &name, BODY, if c.boss { boss_gold() } else { th().gold });
    line_box(ui, "card.name", &name, Rect::new(tx, y, room as i32, 10), BODY);
    let mut hy = y + 10;
    if !c.title.is_empty() {
        ui.p.text_fit(tx as f32, hy as f32, &c.title, SMALL, room, th().dim);
        line_box(ui, "card.title", &c.title, Rect::new(tx, hy, room as i32, 8), SMALL);
        hy += 8;
    }
    // the HP bar previewing the whole party's plan: this hero's share bright, the others' dark
    let (own, others) = c.shares();
    let hb = Rect::new(tx, hy + 1, (room as i32 - 34).max(10), 3);
    ui.p.fill(hb, rgb(30, 20, 26));
    let max = c.read.max_hp.max(1);
    let at = |v: i32| (hb.w * v.clamp(0, max)) / max;
    let hp = c.read.hp.max(0);
    let kept = (hp - own - others).max(0);
    ui.p.fill(Rect::new(hb.x, hb.y, at(kept), hb.h), th().foe);
    let dark = at((hp - own).max(0)) - at(kept);
    ui.p.fill(Rect::new(hb.x + at(kept), hb.y, dark.max(0), hb.h), rgb(110, 40, 40));
    let bright = at(hp) - at((hp - own).max(0));
    ui.p.fill(Rect::new(hb.x + at((hp - own).max(0)), hb.y, bright.max(0), hb.h), rgb(240, 120, 100));
    let hp_text = format!("{}/{}", c.read.hp, c.read.max_hp);
    let hw = ui.p.text_w(&hp_text, SMALL).ceil() as i32;
    ui.p.text_right((r.x + r.w - 5) as f32, hy as f32 - 1.0, &hp_text, SMALL, th().text);
    line_box(ui, "card.hp", &hp_text, Rect::new(r.x + r.w - 5 - hw, hy - 1, hw + 1, 8), SMALL);
    // poise: the pips (the ones the plan breaks in amber), the per-pip tag T
    let py = hy + 7;
    if c.read.max_pips > 0 {
        let broken: i32 = c.read.hits.iter().map(|h| h.pips).sum::<i32>().min(c.read.pips);
        let empties = c.read.break_block.is_some();
        for k in 0..c.read.max_pips {
            let col = if k >= c.read.pips { rgb(40, 48, 64) } else if empties || k >= c.read.pips - broken { planned() } else { super::POISE };
            ui.p.fill(Rect::new(tx + k * 9, py, 8, 3), col);
        }
        let t = format!("T {}", c.read.threshold);
        let tx2 = tx + c.read.max_pips * 9 + 3;
        ui.p.text(tx2 as f32, py as f32 - 2.0, &t, SMALL, super::POISE);
        line_box(ui, "card.threshold", &t, Rect::new(tx2, py - 2, 40, 8), SMALL);
        if empties && !icon(ui, cx, SPLIT_SHIELD, Rect::new(tx - 9, py - 2, 7, 7), 255) {
            ui.p.panel(Rect::new(tx - 9, py - 2, 7, 7), rgba(0, 0, 0, 0), planned(), 1.0);
        }
    }
    // the tags: "KO in block 5" (oxblood, the crit star), "BREAK in block 4" (amber)
    let ty = r.y + HEAD_H - 11;
    let mut tag_x = x;
    let ko = c.ko_text();
    if !ko.is_empty() {
        tag_x += tag(ui, &ko, tag_x, ty, oxblood()) + 2;
        if c.read.crit_star.is_some() {
            icon(ui, cx, CRIT_STAR, Rect::new(tag_x - 10, ty + 1, 7, 7), 255);
        }
    }
    let brk = c.break_text();
    if !brk.is_empty() {
        tag(ui, &brk, tag_x, ty, rgb(0x92, 0x5a, 0x0e));
    }
    // the rows
    let mut ry = r.y + HEAD_H;
    if c.rows.is_empty() {
        let note = if c.hero.is_some() { "No hit planned" } else { "No hits planned" };
        ui.p.text(x as f32, ry as f32 + 1.0, note, SMALL, th().dim);
        line_box(ui, "card.empty", note, Rect::new(x, ry, r.w - 10, ROW_H), SMALL);
    }
    for (k, rw) in c.rows.iter().enumerate() {
        row(ui, cx, small, looks, c, k, rw, x, ry);
        ry += ROW_H;
    }
    // the totals
    let fy = r.y + r.h - FOOT_H + 2;
    ui.p.fill(Rect::new(x, fy - 2, r.w - 10, 1), th().edge);
    let party = c.read.total(None);
    match c.hero {
        Some(h) => {
            let who = looks.get(h).map_or_else(String::new, |l| l.0.name.clone());
            let who = ui.p.fit(&who, SMALL, 38.0);
            foot(ui, "card.hero", &who, x, fy, SMALL, th().text);
            foot(ui, "card.hero_total", &format!("-{own}"), x + 40, fy, BODY, th().gold);
            foot(ui, "card.party", "Party", x, fy + 9, SMALL, th().text);
            foot(ui, "card.party_total", &format!("-{party}"), x + 40, fy + 9, BODY, th().foe);
            if let Some(b) = c.read.ko_block {
                foot(ui, "card.ko", &format!("KO in block {b}"), x + 70, fy + 9, SMALL, th().foe);
            }
        }
        None => {
            foot(ui, "card.party", "Total", x, fy + 4, SMALL, th().text);
            foot(ui, "card.party_total", &format!("-{party}"), x + 40, fy + 4, BODY, th().foe);
        }
    }
}
