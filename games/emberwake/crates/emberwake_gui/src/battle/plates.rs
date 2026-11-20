//! BATTLE-UI2 (PR7-40, `games/emberwake/docs/specs/SCREEN_CONTRACT_BATTLE.md` 3, 4, 6): the battle's plates drawn from
//! the view-model (`battle_view`): the party plates (portrait, name with the round intent tile at the row's end, HP
//! bar with the incoming loss, the AP bar with its balance, poise pips, the status column inside the right edge), the
//! guests' compact plates under their divider, the foe plates (the short name, the intent badge on the portrait, the
//! "when it lands" ruler, the status column inside the left edge, a boss's crown and gold rim), the compact plates
//! from the 7th foe and the groups from the 11th, and the status tabs (icon, buff or debuff rim, dashed when planned,
//! rounds left as pips). Sizes are in the screen's old units (1 unit = 2 px at 1280 x 720).

use super::tips::{intent_glyph, kind_color};
use crate::battle_view::{FoeEntry, Plate, Tab, COMPACT_TABS, STATUS_SLOTS};
use crate::portraits::{Faces, Who};
use crate::screens::Cx;
use crate::tokens::{ap, boss_gold, frame_a, guest, hp, oxblood, planned};
use aegis_ui::canvas::{rgb, rgba, Rect, Rgba};
use aegis_ui::painter::Painter;
use aegis_ui::theme::th;
use aegis_ui::ui::{Tip, Ui, BODY, SMALL};
use emberwake_battle::world::World;

/// Contract 3: a plate's width (176 px), a full plate's height (84 px), a compact one's (52 px), the portrait (48 px),
/// the round intent tile (18 px), the foe's intent badge (16 px), the status column (26 px).
pub(super) const PLATE_W: i32 = 88;
pub(super) const PLATE_H: i32 = 42;
pub(super) const COMPACT_H: i32 = 26;
pub(super) const PORTRAIT: i32 = 24;
pub(super) const INTENT_TILE: i32 = 9;
pub(super) const INTENT_BADGE: i32 = 8;
pub(super) const STATUS_COL: i32 = 13;
/// The gap between plates.
pub(super) const PLATE_GAP: i32 = 2;
/// Contract 4: the "when it lands" ruler's blocks (the round's 6).
const RULER_BLOCKS: i32 = 6;

/// The kit's badge icons (placeholders until the kit icon batch: `kits/rpg/assets/ui/badges/`).
pub(super) const CROWN: &str = "ui/badges/crown.png";
pub(super) const CRIT_STAR: &str = "ui/badges/crit_star.png";
pub(super) const SPLIT_SHIELD: &str = "ui/badges/split_shield.png";
pub(super) const HEART: &str = "ui/badges/heart.png";

/// A unit's portrait and look, read once a frame.
pub(super) type Looks = Vec<(Who, emberwake_battle::portrait::Look)>;

/// `rel` drawn in `r` when the file is there; false when it is not (the caller draws a shape).
pub(super) fn icon(ui: &mut Ui<super::Action>, cx: &mut Cx, rel: &str, r: Rect, alpha: u8) -> bool {
    let px = ui.icon_px(r).max(1);
    match cx.art.image(rel, px, px) {
        Some(img) => {
            ui.p.image_alpha(img, r, alpha);
            true
        }
        None => false,
    }
}

/// Contract 6 (PR7-40 part 2): the moss heart of a heal in `r`: the kit's heart badge, or a drawn moss heart.
pub(super) fn heal_heart(ui: &mut Ui<super::Action>, cx: &mut Cx, r: Rect) {
    if !icon(ui, cx, HEART, r, 255) {
        heart_shape(&mut ui.p, r, crate::tokens::moss());
    }
}

/// A small heart drawn in `r` (two lobes and a point) in `col`.
pub(super) fn heart_shape(p: &mut Painter, r: Rect, col: Rgba) {
    let (x, y, w, h) = (r.x as f32, r.y as f32, r.w as f32, r.h as f32);
    let lobe = w / 4.0;
    p.ellipse(x + lobe, y + lobe, lobe, lobe, col);
    p.ellipse(x + w - lobe, y + lobe, lobe, lobe, col);
    let rows = (h - lobe).ceil() as i32;
    for k in 0..rows.max(1) {
        let t = k as f32 / rows.max(1) as f32;
        let half = (w / 2.0) * (1.0 - t);
        let cxm = x + w / 2.0;
        p.fill(Rect::new((cxm - half).round() as i32, (y + lobe + k as f32).round() as i32, (half * 2.0).round().max(1.0) as i32, 1), col);
    }
}

/// A gold crown on `r` (the boss mark): the badge file, or a drawn crown.
pub(super) fn crown(ui: &mut Ui<super::Action>, cx: &mut Cx, r: Rect) {
    if !icon(ui, cx, CROWN, r, 255) {
        let g = boss_gold();
        ui.p.fill(Rect::new(r.x, r.y + r.h / 2, r.w, r.h / 2), g);
        for k in 0..3 {
            let x = r.x + k * (r.w - 1) / 2;
            ui.p.fill(Rect::new(x, r.y, 1, r.h / 2 + 1), g);
        }
    }
}

/// Contract 6 (PR7-40 part 2): `hp_bar` with the planned heal `gain` past the lit end in pale moss.
fn hp_bar(p: &mut Painter, r: Rect, value: i32, max: i32, col: Rgba, loss: i32, gain: i32) {
    bar(p, r, value, max, col, loss);
    if gain > 0 && max > 0 && r.w > 0 {
        let at = |v: i32| (r.w * v.clamp(0, max) + max - 1) / max;
        let (from, to) = (at(value), at(value + gain));
        if to > from {
            p.fill(Rect::new(r.x + from, r.y, to - from, r.h), crate::tokens::moss_gain());
        }
    }
}

/// A bar in `r` with `value` of `max` lit in `col`, then `loss` of it at the lit end in red (the incoming damage).
fn bar(p: &mut Painter, r: Rect, value: i32, max: i32, col: Rgba, loss: i32) {
    p.fill(r, rgb(30, 20, 26));
    if max <= 0 || r.w <= 0 {
        return;
    }
    let at = |v: i32| (r.w * v.clamp(0, max) + max - 1) / max;
    let (lit, kept) = (at(value), at(value - loss.max(0)));
    p.fill(Rect::new(r.x, r.y, kept, r.h), col);
    if lit > kept {
        p.fill(Rect::new(r.x + kept, r.y, lit - kept, r.h), th().foe);
    }
}

/// Contract 3 (take 21): the AP bar: the cap is its width, the start AP lit in AP blue and the planned part in amber at
/// its end; grey at 0.
fn ap_bar(p: &mut Painter, r: Rect, a: crate::battle_view::ApBar) {
    p.fill(r, rgb(26, 30, 40));
    if a.cap <= 0 {
        return;
    }
    let at = |v: i32| (r.w * v.clamp(0, a.cap) + a.cap - 1) / a.cap;
    let (start, left) = (at(a.start), at(a.left()));
    p.fill(Rect::new(r.x, r.y, left, r.h), ap());
    if start > left {
        p.fill(Rect::new(r.x + left, r.y, start - left, r.h), planned());
    }
}

/// Poise pips (`lit` of `max`) along `r`, one segment each (`marks::poise_pip_rect`).
fn pips(p: &mut Painter, r: Rect, lit: i32, max: i32) {
    let n = max.max(1);
    for k in 0..n {
        p.fill(super::marks::poise_pip_rect(r, k, n), if k < lit { super::POISE } else { rgb(40, 48, 64) });
    }
}

/// Contract 3: a party plate's poise row (the pips along its bottom, left of the status column).
pub(super) fn party_poise_row(r: Rect) -> Rect {
    Rect::new(r.x + 3, r.y + r.h - 4, r.w - STATUS_COL - 6, 2)
}

/// Contract 6: one status tab in `r`: the icon and the rim (gold a buff, oxblood a debuff; dashed when a planned move
/// applies it). BOSSUI1 BU6 (PR7-54): icon only, no pips or numbers; the rounds left are in its list line (`Tab::line`).
pub(super) fn tab(ui: &mut Ui<super::Action>, cx: &mut Cx, t: &Tab, r: Rect) {
    let rim = if t.buff { th().gold } else { oxblood() };
    ui.p.fill(r, rgb(16, 20, 30));
    if t.planned {
        let (x0, y0, x1, y1) = (r.x as f32, r.y as f32, (r.x + r.w) as f32, (r.y + r.h) as f32);
        for (a, b, c, d) in [(x0, y0, x1, y0), (x0, y1, x1, y1), (x0, y0, x0, y1), (x1, y0, x1, y1)] {
            ui.p.line(a, b, c, d, rim, 1.0);
        }
    } else {
        ui.p.panel(r, rgba(0, 0, 0, 0), rim, 0.0);
    }
    let side = (r.h - 2).min(r.w - 2).max(1);
    let ir = Rect::new(r.x + (r.w - side) / 2, r.y + (r.h - side) / 2, side, side);
    if !icon(ui, cx, &t.icon, ir, if t.planned { 170 } else { 255 }) {
        let letter: String = t.name.chars().take(1).collect();
        ui.p.text_center(ir.x as f32 + ir.w as f32 / 2.0, ir.y as f32, &letter, SMALL, rim);
    }
}

/// Contract 3: the plate's status column (4 slots: up to 4 tabs, or 3 and "+N"); each tab a widget with its line.
fn status_column(ui: &mut Ui<super::Action>, cx: &mut Cx, id: &str, tabs: &[Tab], col: Rect, slots: usize) {
    if tabs.is_empty() || slots == 0 {
        return;
    }
    let sh = (col.h / slots as i32).max(4);
    let shown = if tabs.len() > slots { slots - 1 } else { tabs.len() };
    for (k, t) in tabs.iter().take(shown).enumerate() {
        let r = Rect::new(col.x, col.y + k as i32 * sh, col.w, sh - 1);
        tab(ui, cx, t, r);
        ui.push(format!("{id}.status.{k}"), "status", t.line(), r, true, false, None, vec![(t.name.clone(), if t.buff { th().gold } else { th().foe }), (t.line(), th().text)]);
    }
    if tabs.len() > shown {
        let r = Rect::new(col.x, col.y + shown as i32 * sh, col.w, sh - 1);
        ui.p.panel(r, rgb(16, 20, 30), th().edge, 0.0);
        let more = format!("+{}", tabs.len() - shown);
        ui.p.text_center(r.x as f32 + r.w as f32 / 2.0, r.y as f32 + (r.h as f32 - ui.p.line_h(SMALL)) / 2.0, &more, SMALL, th().text);
        // contract 6: the full list on its hover
        let tip: Tip = tabs.iter().map(|t| (t.line(), if t.buff { th().gold } else { th().foe })).collect();
        ui.push(format!("{id}.status.more"), "status", more, r, true, false, None, tip);
    }
}

/// The name in `room`: the body font, the small one when it does not fit, then cut with "..." (contract 3).
fn name_in(p: &mut Painter, x: f32, y: f32, name: &str, room: f32, col: Rgba) {
    let size = if p.text_w(name, BODY) <= room { BODY } else { SMALL };
    p.text_fit(x, y + if size == SMALL { 1.0 } else { 0.0 }, name, size, room, col);
}

/// Contract 3: a party plate in `r` (selected: gold rim; a guest: its compact form).
pub(super) fn party_plate(ui: &mut Ui<super::Action>, cx: &mut Cx, faces: &mut Faces, w: &World, look: &(Who, emberwake_battle::portrait::Look), p: &Plate, r: Rect) {
    let fill = if p.selected { th().card_hot } else if ui.hover(r) { th().game("roster_hot", rgba(28, 40, 66, 255)) } else { th().card };
    frame_a(&mut ui.p, r, fill);
    if p.selected {
        ui.p.panel(r, rgba(0, 0, 0, 0), th().gold, 0.0);
    }
    let pr = Rect::new(r.x + 2, r.y + 3, PORTRAIT, PORTRAIT);
    faces.draw(&mut ui.p, cx, &look.0, &look.1, super::PANEL_SURFACE, pr);
    ui.push(format!("party.{}.portrait", p.unit), "portrait", look.0.name.clone(), pr, true, false, None, Vec::new());
    let col = Rect::new(r.x + r.w - STATUS_COL - 1, r.y + 2, STATUS_COL, r.h - 4);
    let tx = pr.x + pr.w + 2;
    let text_w = col.x - 1 - tx;
    // the name row: the name, the round intent tile at its end
    if let Some(i) = p.intent {
        intent_glyph(&mut ui.p, &w.cat, i, Rect::new(col.x - 1 - INTENT_TILE, r.y + 2, INTENT_TILE, INTENT_TILE));
    }
    let room = (text_w - if p.intent.is_some() { INTENT_TILE + 1 } else { 0 }) as f32;
    name_in(&mut ui.p, tx as f32, r.y as f32 + 2.0, &p.name, room, if p.active { super::ally() } else { th().dim });
    // HP: the bar with the incoming loss, its numbers under it
    hp_bar(&mut ui.p, Rect::new(tx, r.y + 13, text_w, 3), p.hp, p.max_hp, if p.active { hp() } else { th().dim }, p.incoming, p.heal);
    let hp_text = format!("{}/{}", p.hp, p.max_hp);
    let hw = ui.p.text_w(&hp_text, SMALL);
    ui.p.text(tx as f32, r.y as f32 + 17.0, &hp_text, SMALL, th().text);
    let mut nx = tx as f32 + hw + 3.0;
    if p.incoming > 0 {
        let t = format!("-{}", p.incoming);
        ui.p.text(nx, r.y as f32 + 17.0, &t, SMALL, th().foe);
        nx += ui.p.text_w(&t, SMALL) + 2.0;
    }
    // contract 6: a planned heal, the moss heart and "+n"
    if p.heal > 0 {
        let t = format!("+{}", p.heal);
        if nx + 7.0 + ui.p.text_w(&t, SMALL) <= (tx + text_w) as f32 {
            heal_heart(ui, cx, Rect::new(nx as i32, r.y + 17, 6, 6));
            ui.p.text(nx + 7.0, r.y as f32 + 17.0, &t, SMALL, crate::tokens::moss());
            let tw = 7 + ui.p.text_w(&t, SMALL).ceil() as i32;
            ui.push(format!("party.{}.heal", p.unit), "label", t.clone(), Rect::new(nx as i32, r.y + 17, tw, 7), true, false, None, vec![("Planned heal".into(), crate::tokens::moss()), (format!("{t} HP this round"), th().text)]);
        }
    }
    // AP: one bar, the balance after the plan beside it (grey at 0)
    if let Some(a) = p.ap {
        let label = format!("{}/{}", a.left(), a.cap);
        let lw = ui.p.text_w(&label, SMALL).ceil() as i32;
        ap_bar(&mut ui.p, Rect::new(tx, r.y + 27, (text_w - lw - 2).max(4), 3), a);
        ui.p.text_right((tx + text_w) as f32, r.y as f32 + 25.0, &label, SMALL, if a.left() == 0 { th().dim } else { th().text });
    }
    if let Some((lit, max)) = p.poise {
        pips(&mut ui.p, party_poise_row(r), lit, max);
    }
    status_column(ui, cx, &format!("party.{}", p.unit), &p.tabs, col, STATUS_SLOTS);
    passive_row(ui, cx, &format!("party.{}", p.unit), &p.passives, Rect::new(pr.x, pr.y + pr.h + 1, pr.w, PASSIVE_ICON));
}

/// PASSIVE-ICON1 (ENGINE-24, STATUSVIS1 SV10): a passive icon under the portrait (16 px at 1280 x 720) and the gap.
pub(super) const PASSIVE_ICON: i32 = 8;
const PASSIVE_GAP: i32 = 2;
/// SV12: the placeholder while an archetype's icon is not there yet (a skill kind's icon).
const PASSIVE_PLACEHOLDER: &str = "skills/kinds/focus.png";

/// SV10: the loadout's passives as small icons in `row` under the portrait (apart from the status column), each its
/// own label `<id>.passive.<i>` with the passive's name on its hover card.
fn passive_row(ui: &mut Ui<super::Action>, cx: &mut Cx, id: &str, passives: &[crate::battle_view::PlatePassive], row: Rect) {
    for (i, p) in passives.iter().enumerate() {
        let r = Rect::new(row.x + i as i32 * (PASSIVE_ICON + PASSIVE_GAP), row.y, PASSIVE_ICON, PASSIVE_ICON);
        if r.x + r.w > row.x + row.w {
            break;
        }
        let px = ui.icon_px(r).max(1);
        let img = cx.art.first(&[p.icon.clone(), PASSIVE_PLACEHOLDER.to_string()], px, px).cloned();
        aegis_ui::panes::icon_tile(ui, r, img.as_ref(), if img.is_none() { "P" } else { "" });
        ui.push(format!("{id}.passive.{i}"), "label", p.name.clone(), r, true, false, None, vec![(p.name.clone(), th().gold), ("Loadout passive: always on.".into(), th().text)]);
    }
}

/// Contract 3: a guest's compact plate (teal rim, "AI" tag, HP, intent, a gold "Protect" tag).
pub(super) fn guest_plate(ui: &mut Ui<super::Action>, cx: &mut Cx, faces: &mut Faces, w: &World, look: &(Who, emberwake_battle::portrait::Look), p: &Plate, r: Rect) {
    frame_a(&mut ui.p, r, th().card);
    ui.p.panel(r, rgba(0, 0, 0, 0), guest(), 0.0);
    let pr = Rect::new(r.x + 2, r.y + 2, r.h - 4, r.h - 4);
    faces.draw(&mut ui.p, cx, &look.0, &look.1, super::PANEL_SURFACE, pr);
    let tx = pr.x + pr.w + 2;
    let right = r.x + r.w - 2;
    if let Some(i) = p.intent {
        intent_glyph(&mut ui.p, &w.cat, i, Rect::new(right - INTENT_TILE, r.y + 2, INTENT_TILE, INTENT_TILE));
    }
    name_in(&mut ui.p, tx as f32, r.y as f32 + 2.0, &p.name, (right - INTENT_TILE - 1 - tx) as f32, if p.active { super::ally() } else { th().dim });
    hp_bar(&mut ui.p, Rect::new(tx, r.y + 12, right - tx, 2), p.hp, p.max_hp, if p.active { hp() } else { th().dim }, p.incoming, p.heal);
    let mut x = tx as f32;
    for (label, col) in [("AI", guest())].into_iter().chain(p.protect.then_some(("Protect", th().gold))) {
        let lw = ui.p.text_w(label, SMALL) + 4.0;
        let tr = Rect::new(x as i32, r.y + 16, lw.ceil() as i32, 8);
        ui.p.panel(tr, rgba(0, 0, 0, 0), col, 1.0);
        ui.p.text(x + 2.0, tr.y as f32 + 0.5, label, SMALL, col);
        x += lw + 2.0;
    }
}

/// Contract 4: the "when it lands" row in `r`: the blow's kind glyph, its target's face and a 6-block ruler with the
/// impact block in red.
fn lands_row(ui: &mut Ui<super::Action>, cx: &mut Cx, small: &mut Faces, looks: &Looks, l: &crate::battle_view::Lands, r: Rect) {
    ui.p.fill(Rect::new(r.x, r.y + 1, r.h - 2, r.h - 2), kind_color(&l.kind));
    let mut x = r.x + r.h;
    if let Some(t) = l.target.and_then(|t| looks.get(t)) {
        small.draw(&mut ui.p, cx, &t.0, &t.1, super::TIMELINE_SURFACE, Rect::new(x, r.y, r.h, r.h));
        x += r.h + 1;
    }
    let seg = ((r.x + r.w - x) / RULER_BLOCKS).max(2);
    for b in 1..=RULER_BLOCKS {
        let br = Rect::new(x + (b - 1) * seg, r.y + r.h / 2 - 1, seg - 1, 2);
        ui.p.fill(br, if b == l.impact { th().foe } else { rgb(70, 78, 96) });
    }
}

/// Contract 4: a foe's full plate in `r`: the status column inside the left edge, the portrait with the intent badge
/// (a boss's crown), the short name (gold for a boss), the HP bar and numbers, the "when it lands" row (its target's
/// face from `small`, the capped faces), poise pips.
#[allow(clippy::too_many_arguments)]
pub(super) fn foe_plate(ui: &mut Ui<super::Action>, cx: &mut Cx, faces: &mut Faces, small: &mut Faces, w: &World, looks: &Looks, p: &Plate, r: Rect, hot: bool) {
    frame_a(&mut ui.p, r, if hot { th().game("foe_hot", rgba(48, 28, 38, 255)) } else { th().card });
    if p.boss {
        ui.p.panel(r, rgba(0, 0, 0, 0), boss_gold(), 0.0);
    } else if hot {
        ui.p.panel(r, rgba(0, 0, 0, 0), th().foe, 0.0);
    }
    let col = Rect::new(r.x + 1, r.y + 2, STATUS_COL, r.h - 4);
    let pr = Rect::new(col.x + col.w + 1, r.y + 3, PORTRAIT, PORTRAIT);
    if let Some(l) = looks.get(p.unit) {
        faces.draw(&mut ui.p, cx, &l.0, &l.1, super::PANEL_SURFACE, pr);
        ui.push(format!("foe.{}.portrait", p.unit), "portrait", l.0.name.clone(), pr, true, false, None, Vec::new());
    }
    if let Some(i) = p.intent {
        intent_glyph(&mut ui.p, &w.cat, i, Rect::new(pr.x + pr.w - INTENT_BADGE, pr.y + pr.h - INTENT_BADGE, INTENT_BADGE, INTENT_BADGE));
    }
    if p.boss {
        crown(ui, cx, Rect::new(pr.x - 1, pr.y - 2, 8, 6));
    }
    let tx = pr.x + pr.w + 2;
    let text_w = r.x + r.w - 2 - tx;
    let name_col = if !p.active { th().dim } else if p.boss { boss_gold() } else { th().text };
    name_in(&mut ui.p, tx as f32, r.y as f32 + 2.0, &p.name, text_w as f32, name_col);
    bar(&mut ui.p, Rect::new(tx, r.y + 13, text_w, 3), p.hp, p.max_hp, if p.active { th().foe } else { th().dim }, p.incoming);
    let hp_text = if p.active { format!("{}/{}", p.hp, p.max_hp) } else { "Down".to_string() };
    ui.p.text(tx as f32, r.y as f32 + 17.0, &hp_text, SMALL, if p.active { th().text } else { th().dim });
    if let Some(l) = p.lands.as_ref().filter(|_| p.active) {
        lands_row(ui, cx, small, looks, l, lands_rect(r));
    }
    if let Some((lit, max)) = p.poise {
        pips(&mut ui.p, Rect::new(pr.x, r.y + r.h - 4, r.x + r.w - 3 - pr.x, 2), lit, max);
    }
    status_column(ui, cx, &format!("foe.{}", p.unit), &p.tabs, col, STATUS_SLOTS);
}

/// Contract 4: where a full foe plate in `r` draws its "when it lands" row.
pub(super) fn lands_rect(r: Rect) -> Rect {
    let tx = r.x + 1 + STATUS_COL + 1 + PORTRAIT + 2;
    Rect::new(tx, r.y + 26, r.x + r.w - 2 - tx, 6)
}

/// Contract 4: a compact foe plate (from the 7th foe): name, HP, poise and up to 2 status tabs.
pub(super) fn compact_plate(ui: &mut Ui<super::Action>, cx: &mut Cx, p: &Plate, r: Rect, hot: bool) {
    frame_a(&mut ui.p, r, if hot { th().game("foe_hot", rgba(48, 28, 38, 255)) } else { th().card });
    if p.boss {
        ui.p.panel(r, rgba(0, 0, 0, 0), boss_gold(), 0.0);
    }
    let tabs_w = COMPACT_TABS as i32 * (STATUS_COL + 1);
    let tx = r.x + 3;
    let text_w = r.w - 6 - tabs_w;
    name_in(&mut ui.p, tx as f32, r.y as f32 + 2.0, &p.name, text_w as f32, if !p.active { th().dim } else if p.boss { boss_gold() } else { th().text });
    bar(&mut ui.p, Rect::new(tx, r.y + 12, text_w, 3), p.hp, p.max_hp, if p.active { th().foe } else { th().dim }, p.incoming);
    ui.p.text(tx as f32, r.y as f32 + 15.0, &format!("{}/{}", p.hp, p.max_hp), SMALL, th().text);
    if let Some((lit, max)) = p.poise {
        pips(&mut ui.p, Rect::new(tx, r.y + r.h - 3, text_w, 1), lit, max);
    }
    let col = Rect::new(r.x + r.w - 2 - tabs_w, r.y + 3, STATUS_COL, r.h - 6);
    for (k, t) in p.tabs.iter().take(COMPACT_TABS).enumerate() {
        let tr = Rect::new(col.x + k as i32 * (STATUS_COL + 1), col.y, STATUS_COL, 9);
        tab(ui, cx, t, tr);
    }
}

/// Contract 4: a group row (from the 11th foe): stacked portraits, "Ash-Crawler x4", one thin HP bar each.
pub(super) fn group_row(ui: &mut Ui<super::Action>, cx: &mut Cx, small: &mut Faces, looks: &Looks, name: &str, plates: &[Plate], r: Rect) {
    frame_a(&mut ui.p, r, th().card);
    for (k, p) in plates.iter().take(3).enumerate().rev() {
        if let Some(l) = looks.get(p.unit) {
            small.draw(&mut ui.p, cx, &l.0, &l.1, super::TIMELINE_SURFACE, Rect::new(r.x + 2 + k as i32 * 3, r.y + 2 + k as i32 * 2, 16, 16));
        }
    }
    let tx = r.x + 26;
    ui.p.text_fit(tx as f32, r.y as f32 + 2.0, name, SMALL, (r.x + r.w - 2 - tx) as f32, th().text);
    let n = plates.len().max(1) as i32;
    let bh = ((r.h - 12) / n).clamp(1, 3);
    for (k, p) in plates.iter().enumerate() {
        bar(&mut ui.p, Rect::new(tx, r.y + 11 + k as i32 * (bh + 1), r.x + r.w - 3 - tx, bh), p.hp, p.max_hp, th().foe, p.incoming);
    }
}

/// Contract 4 (PR7-40 part 2): an open group's slim header ("Ash-Crawler x4" and an up chevron; a click closes it).
pub(super) fn group_header(ui: &mut Ui<super::Action>, name: &str, r: Rect) {
    frame_a(&mut ui.p, r, th().card);
    let tx = r.x + 3;
    ui.p.text_fit(tx as f32, r.y as f32 + (r.h as f32 - ui.p.line_h(SMALL)) / 2.0, name, SMALL, (r.w - 14) as f32, th().text);
    let (cx, cy) = ((r.x + r.w - 6) as f32, r.y as f32 + r.h as f32 / 2.0);
    ui.p.line(cx - 2.5, cy + 1.2, cx, cy - 1.3, th().text, 0.0);
    ui.p.line(cx, cy - 1.3, cx + 2.5, cy + 1.2, th().text, 0.0);
}

/// Contract 4: an open group's header height.
pub(super) const GROUP_HEAD_H: i32 = 10;

/// Contract 4: the height of a foe panel row.
pub(super) fn entry_h(e: &FoeEntry) -> i32 {
    match e {
        FoeEntry::Full(_) => PLATE_H,
        FoeEntry::Group { open: true, .. } => GROUP_HEAD_H,
        _ => COMPACT_H,
    }
}
