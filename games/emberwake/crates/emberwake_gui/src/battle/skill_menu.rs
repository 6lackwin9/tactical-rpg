//! VISCLAR1-D (mock `menu_icons`, decision "Battle view answers, part 2"): the skill menu. A neutral row (Fill, Move, Rest)
//! on top, then the rows of each intent the ruleset lists. Every place comes from the menu's anchor and the rows the kit
//! lists, and that list does not change with the picks, so when an opener fixes the intent the other rows fade where they
//! stand. Sizes are in the wrapper's units, as the rest of this screen; the measured columns grow with the text size.
//! INTENT2-U (IR19, Option A): a pouch item is a row in each intent it allows, after the skills, with its live copies as
//! an `xN` badge on its icon (its card has the cost). BATTLE-UI2 (PR7-40 part 3, battle contract 7, FB-358): one row per
//! skill replaces the icon grid (one skill menu everywhere): icon, name, AP pips and block span, HP and poise damage on
//! the target, status chips; it opens on a strip block, on a board target, and on a ghost block from a right click or
//! the menu key (Tab, pad north).

use super::board::with_alpha;
use super::input::{Popup, Row};
use super::tips::{icon_line, intent_ring_color, text_outlined};
use super::Action;
use crate::screens::Cx;
use aegis_ui::canvas::{rgb, rgba, Rect, Rgba};
use aegis_ui::painter::Painter;
use aegis_ui::theme::th;
use aegis_ui::ui::{Ui, BODY, SMALL};
use emberwake_battle::round_intent::{ManeuverKind, RoundIntent};
use emberwake_battle::round_intent_plan as rip;
use emberwake_battle::world::World;

/// A skill cell's side.
pub(super) const ICON: i32 = 15;
/// The gap between cells and lines.
const GAP: i32 = 2;
/// The panel's inner margin.
const PAD: i32 = 4;
/// The narrowest panel, so the title has room.
const MENU_MIN_W: i32 = 170;
/// A skill cell's edge (the mock's light frame).
const CELL_EDGE: Rgba = rgb(170, 175, 190);
/// A cell that cannot be picked now, and a row whose intent another fixed: how much of it shows.
const FADE: u8 = 70;
/// The neutral row's name.
const NEUTRAL_LINE: &str = "Any time";
/// The neutral row's buttons in their places (the mock's Fill, Move, Rest; on another ally, Select). PK1-S2c: the
/// Channel toggle (`CHANNEL`) goes after Rest only on a menu that has it, so every other menu keeps its places and width.
const NEUTRAL: [&str; 4] = ["Fill", "Move", "Rest", "Select"];
/// PK1-S2c (PK1 4.3 item 1, user 2026-11-10): the Channel toggle's button (0 AP, 0 blocks; only for a unit that can
/// Channel). A placeholder place: the final UI moves it to a mode control.
pub(super) const CHANNEL: &str = "Channel";
/// PK1-S2c (PK1 4.3 item 6): the placeholder Channel look (the toggle lit, a Channel round's unit and its marrow
/// blocks): a violet tint until the UI layout work designs the real one (user 2026-11-10).
pub(crate) const CHANNEL_TINT: Rgba = rgb(170, 90, 230);
/// How strongly the placeholder tint lies over a unit's sprite.
pub(super) const CHANNEL_TINT_ALPHA: u8 = 110;

/// GEAR-REWORK1 (PR5-51a pass 2, `docs/sessions/U3_GEAR_REWORK.md` 4): the Off-Hand button's place (only on a menu whose
/// hero has an off-hand action, `offhand::action`); the button shows the action's name (Reload, Fade-Step, a jab).
pub(super) const OFF_HAND: &str = "Off-Hand";

/// The neutral buttons of a menu by sub-line: Fill, Move, Rest (plus Select on another ally) on line 0, and
/// optional Channel / Off-Hand controls on a second neutral line so added off-hand names never widen line 0.
fn neutral_lines(rows: &[Row]) -> Vec<Vec<&'static str>> {
    let has = |t: &'static str| rows.iter().any(|r| r.slot == Some(Slot::Neutral(t)));
    let line0: Vec<&'static str> = NEUTRAL.iter().copied().filter(|&t| has(t)).collect();
    let line1: Vec<&'static str> = [CHANNEL, OFF_HAND].into_iter().filter(|&t| has(t)).collect();
    if line1.is_empty() { vec![line0] } else { vec![line0, line1] }
}

/// Where a row of the menu goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Slot {
    /// A button of the neutral row, by its text.
    Neutral(&'static str),
    /// A cell in that intent's row.
    Intent(RoundIntent),
}

/// The intents the battle offers, in `RoundIntent::ALL`'s order: the menu's rows. A fifth intent costs one entry there.
/// A4.2-K1b: the ruleset's listed intents, and Field under the rule `field_intent` (`round_intent::listed`).
pub(super) fn menu_intents(w: &World) -> Vec<RoundIntent> {
    RoundIntent::ALL.into_iter().filter(|i| emberwake_battle::round_intent::listed(w, *i)).collect()
}

/// BATTLE-UI2 (battle contract 7): a skill row's facts: its AP and blocks, what it lands on its target (the kit's
/// `pick_hits`; None off a foe or refused), whether it heals and the statuses it brings (the strip's chips).
#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct RowFacts {
    pub(super) ap: i32,
    pub(super) blocks: i32,
    pub(super) hits: Option<emberwake_battle::round_intent_plan::PickHits>,
    pub(super) heal: bool,
    pub(super) statuses: Vec<String>,
}

/// BATTLE-UI2 (battle contract 7): a skill row's height, its icon's side, the damage and poise columns, a status chip.
pub(super) const ROW_H: i32 = 17;
const ROW_ICON: i32 = 14;
const DAMAGE_W: i32 = 30;
const POISE_W: i32 = 24;
const CHIP: i32 = 8;
/// The chips a row shows (then "+N"), and the widest name column.
const ROW_CHIPS: usize = 3;
const NAME_MAX_W: i32 = 104;
/// The footer's hint (contract 7: hover lights the move's blocks on the strip) and its height.
pub(super) const MENU_HINT: &str = "Hover a row: its blocks light on the timeline";
const FOOT_H: i32 = 9;

/// The layout of a skill menu (BATTLE-UI2 contract 7: one row per skill, replacing VISCLAR1-D's icon grid; the neutral
/// buttons on a line under the title). `places[k]`: popup row `k`'s line and column; the neutral lines come first (a
/// button each), then one line per skill row, the rows of each intent together in the ruleset's order.
#[derive(Debug, Clone)]
pub(super) struct MenuGrid {
    /// Each intent's name and the first line its rows take (a neutral line: None).
    pub(super) lines: Vec<(String, Option<RoundIntent>, i32)>,
    pub(super) places: Vec<(i32, i32)>,
    /// How many lines in all (the neutral lines and the skill rows).
    pub(super) depth: i32,
    /// The round's intent once a pick holds it: the other intents' rows fade.
    pub(super) fixed: Option<RoundIntent>,
    /// The intent rows drawn and clickable. Every listed row keeps its place whether it shows or not, so showing a subset is
    /// this one list and nothing shifts. Every listed intent today.
    pub(super) shown: Vec<RoundIntent>,
    /// The name column, measured for the text size (`fit`).
    pub(super) label_w: i32,
    pub(super) neutral: Vec<Vec<&'static str>>,
    pub(super) neutral_ws: Vec<Vec<i32>>,
    pub(super) rest_name: String,
    /// GEAR-REWORK1 (PR5-51a pass 2): the Off-Hand button's text (its action's name; measured with the buttons).
    pub(super) off_name: String,
    /// The skill rows' names, measured for the name column.
    names: Vec<String>,
}

impl MenuGrid {
    /// The layout of `rows` under the intents `intents` (each with its name; every intent the ruleset lists, so the places
    /// do not depend on `shown`): the neutral buttons on their lines, then each intent's rows in their order, one a line.
    pub(super) fn new(intents: &[(RoundIntent, String)], shown: Vec<RoundIntent>, rows: &[Row], fixed: Option<RoundIntent>) -> Self {
        let mut lines = vec![(NEUTRAL_LINE.to_string(), None, 0)];
        let mut places = vec![(0, 0); rows.len()];
        let neutral = neutral_lines(rows);
        let neutral_ws = neutral.iter().map(|line| vec![ICON + 7; line.len()]).collect();
        let mut depth = neutral.len() as i32;
        for (k, row) in rows.iter().enumerate() {
            if let Some(Slot::Neutral(text)) = row.slot {
                if let Some((l, c)) = neutral.iter().enumerate().find_map(|(l, line)| line.iter().position(|n| *n == text).map(|c| (l as i32, c as i32))) {
                    places[k] = (l, c);
                }
            }
        }
        let mut names = Vec::new();
        for (i, name) in intents {
            lines.push((name.clone(), Some(*i), depth));
            for k in (0..rows.len()).filter(|&k| rows[k].slot == Some(Slot::Intent(*i))) {
                places[k] = (depth, 0);
                names.push(rows[k].label.clone());
                depth += 1;
            }
        }
        let rest_name = rows.iter().find(|r| r.slot == Some(Slot::Neutral("Rest"))).map(|r| r.label.clone()).filter(|l| l != "Rest").unwrap_or_default();
        let off_name = rows.iter().find(|r| r.slot == Some(Slot::Neutral(OFF_HAND))).map(|r| r.label.clone()).unwrap_or_default();
        MenuGrid { lines, places, depth, fixed, shown, label_w: 60, neutral, neutral_ws, rest_name, off_name, names }
    }

    /// Measures the name column and each neutral button individually for the text size.
    pub(super) fn fit(&mut self, p: &mut Painter) {
        let label = self.names.iter().map(|l| p.text_w(l, BODY)).fold(0.0, f32::max);
        self.label_w = (label.ceil() as i32 + 2).clamp(40, NAME_MAX_W);
        self.neutral_ws = self.neutral.iter().map(|line| {
            line.iter().map(|&b| {
                let text = if b == "Rest" && !self.rest_name.is_empty() {
                    self.rest_name.as_str()
                } else if b == OFF_HAND && !self.off_name.is_empty() {
                    self.off_name.as_str()
                } else {
                    b
                };
                (p.text_w(text, SMALL).ceil() as i32 + 6).max(ICON)
            }).collect()
        }).collect();
    }

    /// The neutral lines' count.
    fn neutral_n(&self) -> i32 {
        self.neutral.len() as i32
    }

    /// A skill row's width: icon, name column, damage, poise and chips.
    fn row_w(&self) -> i32 {
        ROW_ICON + 4 + self.label_w + 4 + DAMAGE_W + POISE_W + ROW_CHIPS as i32 * (CHIP + 1) + 4
    }

    /// The panel's size with a title row `row_h` high (the footer's hint under the rows).
    pub(super) fn size(&self, row_h: i32) -> (i32, i32) {
        let neutral = self.neutral_ws.iter().map(|line| line.iter().map(|&w| w + GAP).sum::<i32>() - GAP).max().unwrap_or(0);
        let rows = self.depth - self.neutral_n();
        let h = row_h + 2 + self.neutral_n() * (ICON + GAP) + rows * (ROW_H + 1) + FOOT_H + PAD;
        ((2 * PAD + neutral.max(self.row_w())).max(MENU_MIN_W), h)
    }

    /// Row `k`'s rect in a panel at (`x`, `y`): a neutral button, or a skill row across the panel.
    pub(super) fn cell(&self, x: i32, y: i32, row_h: i32, k: usize) -> Rect {
        let (line, col) = self.places.get(k).copied().unwrap_or((0, 0));
        let left = x + PAD;
        let top = y + row_h + 2;
        if let Some(ws) = usize::try_from(line).ok().filter(|l| *l < self.neutral.len()).and_then(|l| self.neutral_ws.get(l)) {
            let off: i32 = ws.iter().take(col as usize).map(|&w| w + GAP).sum();
            let bw = ws.get(col as usize).copied().unwrap_or(ICON + 7);
            Rect::new(left + off, top + line * (ICON + GAP), bw, ICON)
        } else {
            let (w, _) = self.size(row_h);
            let r = line - self.neutral_n();
            Rect::new(left, top + self.neutral_n() * (ICON + GAP) + r * (ROW_H + 1), w - 2 * PAD, ROW_H)
        }
    }

    /// Whether a row of the menu in `slot` is drawn and takes clicks (the neutral row always).
    pub(super) fn shows(&self, slot: Option<Slot>) -> bool {
        !matches!(slot, Some(Slot::Intent(i)) if !self.shown.contains(&i))
    }

    /// Whether `i`'s rows fade: another intent fixed the round.
    pub(super) fn faded(&self, i: Option<RoundIntent>) -> bool {
        matches!((self.fixed, i), (Some(f), Some(i)) if f != i)
    }
}

/// VISCLAR1-D (user 2026-11-05): where a menu of `size` opens beside tile `cell`: to its right, to its left near the right edge,
/// centred on it up and down, inside the battle view `view`, so it never covers the strip or leaves the screen.
pub(super) fn menu_place(cell: Rect, size: (i32, i32), view: Rect) -> (i32, i32) {
    let (w, h) = size;
    let right = cell.x + cell.w + GAP;
    let left = cell.x - GAP - w;
    let x = if right + w <= view.x + view.w { right } else if left >= view.x { left } else { (view.x + view.w - w).max(view.x) };
    let y = (cell.y + cell.h / 2 - h / 2).clamp(view.y, (view.y + view.h - h).max(view.y));
    (x, y)
}

/// BATTLE-UI2 (battle contract 7): where a menu of `size` opens on strip block `block`: under it, its left edge on the
/// block's, inside the screen `view`.
pub(super) fn menu_below(block: Rect, size: (i32, i32), view: Rect) -> (i32, i32) {
    let (w, h) = size;
    let x = block.x.clamp(view.x, (view.x + view.w - w).max(view.x));
    let y = (block.y + block.h + GAP + 2).clamp(view.y, (view.y + view.h - h).max(view.y));
    (x, y)
}

/// Draws the menu: panel, title, the neutral buttons, one row per skill (contract 7: icon, name, AP pips and block span,
/// HP and poise damage on the target, status chips) and the footer's hint, each with its widget and hover card.
pub(super) fn draw_menu(ui: &mut Ui<Action>, cx: &mut Cx, pop: &Popup, grid: &MenuGrid, row_h: i32) {
    let r = pop.rect(row_h);
    // BATTLE-UI2 contract 1: the menu wears frame A
    crate::tokens::frame_a(&mut ui.p, r, th().game("popup_fill", rgba(14, 20, 34, 252)));
    // a near miss in a gap keeps the menu open
    ui.push("popup.panel", "panel", "", r, true, false, Some(Action::MenuGap), Vec::new());
    let title = ui.p.fit(&pop.title, BODY, r.w as f32 - 10.0);
    ui.p.text_face(aegis_ui::painter::Typeface::Display, r.x as f32 + 5.0, r.y as f32 + 2.0, &title, BODY, th().gold);
    let hint_y = r.y + r.h - PAD - FOOT_H + 1;
    ui.p.text_fit(r.x as f32 + 5.0, hint_y as f32, MENU_HINT, SMALL, r.w as f32 - 10.0, th().dim);
    // a hairline between two intents' rows (each intent's first line after the first)
    let firsts: Vec<i32> = grid.lines.iter().filter(|l| l.1.is_some_and(|i| grid.shown.contains(&i))).map(|l| l.2).collect();
    for line in firsts.iter().skip(1).filter(|l| **l < grid.depth && pop.rows.iter().enumerate().any(|(k, _)| grid.places.get(k).map(|p| p.0) == Some(**l))) {
        let y = r.y + row_h + 2 + grid.neutral.len() as i32 * (ICON + GAP) + (line - grid.neutral.len() as i32) * (ROW_H + 1) - 1;
        ui.p.fill(Rect::new(r.x + PAD, y, r.w - 2 * PAD, 1), with_alpha(th().edge, 160));
    }
    for (k, row) in pop.rows.iter().enumerate().filter(|(_, r)| grid.shows(r.slot)) {
        let rr = pop.row(row_h, k);
        let hover = ui.hover(rr);
        let faded = matches!(row.slot, Some(Slot::Intent(i)) if grid.faded(Some(i)));
        let shows = if row.valid && !faded { 255 } else { FADE };
        let lit = hover && row.valid;
        match row.slot {
            Some(Slot::Neutral(text)) => {
                // PK1-S2c: the Channel toggle while on is lit in the placeholder Channel tint
                let on = matches!(row.act, super::input::RowAct::Channel(false));
                let fill = if lit { th().card_hot } else if on { CHANNEL_TINT } else { th().card };
                let edge = if lit { th().gold } else if on { CHANNEL_TINT } else { th().edge_live };
                ui.p.panel(rr, with_alpha(fill, shows), with_alpha(edge, shows), 1.5);
                let y = rr.y as f32 + (rr.h as f32 - ui.p.line_h(SMALL)) / 2.0;
                // PK1-S2c (PK1 4.3 item 5): the Rest button names what a Rest block plans (Feed in a Channel round)
                // GEAR-REWORK1 (PR5-51a pass 2): the Off-Hand button names its action (Reload, Fade-Step, a jab)
                let shown = if (text == "Rest" || text == OFF_HAND) && !row.label.is_empty() { row.label.as_str() } else { text };
                ui.p.text_center(rr.x as f32 + rr.w as f32 / 2.0, y, shown, SMALL, with_alpha(th().text, shows.max(110)));
            }
            _ => skill_row(ui, cx, row, rr, grid.label_w, shows, lit),
        }
        // IF3, IN6: a pick that changes the intent and drops intent-bound buffs: a red corner
        if row.warn {
            ui.p.fill(Rect::new(rr.x + rr.w - 4, rr.y + 1, 3, 3), with_alpha(th().foe, shows));
        }
        ui.push(format!("popup.{k}"), "option", format!("{} {}", row.label, row.detail), rr, row.valid, hover, Some(Action::Choose(k)), row.tip.clone());
    }
}

/// One skill row in `rr` (contract 7): the icon with its intent line, the name over its AP pips and block span, the HP
/// damage and the poise pips on the target, the status chips; `shows` its alpha, `lit` hovered and live.
fn skill_row(ui: &mut Ui<Action>, cx: &mut Cx, row: &Row, rr: Rect, name_w: i32, shows: u8, lit: bool) {
    if lit {
        ui.p.panel(rr, th().card_hot, th().gold, 1.5);
    }
    let ir = Rect::new(rr.x + 2, rr.y + (ROW_H - ROW_ICON) / 2, ROW_ICON, ROW_ICON);
    ui.p.panel(ir, with_alpha(th().card, shows), with_alpha(if lit { th().gold } else { CELL_EDGE }, shows), 1.0);
    let px = ui.icon_px(ir).max(1);
    let rel = cx.art.first_rel(&row.icon);
    if let Some(img) = rel.as_deref().and_then(|rel| cx.art.image(rel, px, px)) {
        ui.p.image_alpha(img, Rect::new(ir.x + 1, ir.y + 1, ir.w - 2, ir.h - 2), shows);
    }
    // PR7-36 (ART-12): the row's intent as a thin bottom line on the icon
    if let Some(Slot::Intent(i)) = row.slot {
        icon_line(&mut ui.p, Rect::new(ir.x + 1, ir.y + 1, ir.w - 2, ir.h - 2), intent_ring_color(i), shows);
    }
    // the second cell of a skill with both Maneuver kinds: Disengage (its card says so)
    if matches!(&row.act, super::input::RowAct::Pick(p) if matches!(p[..], [rip::Pick::Skill { kind: Some(ManeuverKind::Disengage), .. }])) {
        ui.p.fill(Rect::new(ir.x + ir.w - 5, ir.y + ir.h - 5, 4, 4), with_alpha(th().text, shows));
    }
    // a pouch item's copies left (INTENT2-U: the corner `xN` badge)
    if let Some(n) = row.count {
        let text = format!("x{n}");
        let (tw, lh) = (ui.p.text_w(&text, SMALL), ui.p.line_h(SMALL));
        text_outlined(&mut ui.p, (ir.x + ir.w) as f32 - tw, (ir.y + ir.h) as f32 - lh + 1.0, &text, SMALL, with_alpha(th().text, shows.max(150)), rgba(0, 0, 0, shows));
    }
    let nx = ir.x + ir.w + 4;
    let name = ui.p.fit(&row.label, BODY, name_w as f32);
    ui.p.text(nx as f32, rr.y as f32, &name, BODY, with_alpha(th().text, shows));
    let Some(f) = &row.facts else { return };
    // the AP pips (blue), then the block span (grey), under the name
    let py = rr.y + ROW_H - 5;
    let mut at = nx;
    for _ in 0..f.ap.clamp(0, 12) {
        ui.p.fill(Rect::new(at, py, 3, 3), with_alpha(crate::tokens::ally(), shows));
        at += 4;
    }
    at += if f.ap > 0 { 3 } else { 0 };
    for _ in 0..f.blocks.clamp(0, 6) {
        ui.p.fill(Rect::new(at, py + 1, 3, 2), with_alpha(CELL_EDGE, shows));
        at += 4;
    }
    let mut x = nx + name_w + 4;
    if let Some(h) = f.hits.filter(|h| h.hits > 0) {
        super::plates::heart_shape(&mut ui.p, Rect::new(x, rr.y + 5, 6, 5), with_alpha(th().foe, shows));
        ui.p.text(x as f32 + 8.0, rr.y as f32 + 3.0, &format!("-{}", h.damage), BODY, with_alpha(th().text, shows));
        let dx = x + DAMAGE_W;
        if h.breaks {
            let sr = Rect::new(dx, rr.y + 4, 7, 7);
            if !super::plates::icon(ui, cx, super::plates::SPLIT_SHIELD, sr, shows) {
                ui.p.panel(sr, rgba(0, 0, 0, 0), with_alpha(crate::tokens::planned(), shows), 1.0);
            }
        } else {
            for k in 0..h.pips.clamp(0, 5) {
                ui.p.fill(Rect::new(dx + k * 4, rr.y + 6, 3, 3), with_alpha(crate::tokens::planned(), shows));
            }
        }
    }
    x += DAMAGE_W + POISE_W;
    // the heal's moss heart, then each status the move brings (at most ROW_CHIPS, then "+N")
    let mut chips: Vec<String> = Vec::new();
    if f.heal {
        chips.push(String::new());
    }
    chips.extend(f.statuses.iter().cloned());
    for st in chips.iter().take(ROW_CHIPS) {
        let cr = Rect::new(x, rr.y + (ROW_H - CHIP) / 2, CHIP, CHIP);
        if st.is_empty() {
            super::plates::heal_heart(ui, cx, cr);
        } else if !super::plates::icon(ui, cx, &emberwake_content::art::status_icon(st), cr, shows) {
            ui.p.panel(cr, rgba(0, 0, 0, 0), with_alpha(crate::tokens::oxblood(), shows), 1.0);
        }
        x += CHIP + 1;
    }
    if chips.len() > ROW_CHIPS {
        ui.p.text(x as f32, rr.y as f32 + 4.0, &format!("+{}", chips.len() - ROW_CHIPS), SMALL, with_alpha(th().text, shows));
    }
}
