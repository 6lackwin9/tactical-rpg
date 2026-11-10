//! VISCLAR1-D (mock `menu_icons`, decision "Battle view answers, part 2"): the skill menu as icons only. A neutral row (Fill,
//! Move, Rest) on top, then one row per intent the ruleset lists. Every place comes from the menu's anchor and the rows the
//! kit lists, and that list does not change with the picks, so when an opener fixes the intent the other rows fade where
//! they stand. Sizes are in the wrapper's units, as the rest of this screen; the measured columns grow with the text size.
//! INTENT2-U (IR19, Option A): every intent row on one line, no block count on a cell; a pouch item is an icon cell in
//! each row it allows, after the skills, with its live copies as an `xN` badge in the corner (its card has the cost).

use super::board::with_alpha;
use super::input::{Popup, Row};
use super::tips::{intent_ring_color, text_outlined};
use super::Action;
use aegis_ui::art::Art;
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

/// The neutral buttons of a menu in their order: Fill, Move, Rest, the Channel toggle when `channel`, Select.
fn neutral_order(channel: bool) -> Vec<&'static str> {
    let mut out = vec![NEUTRAL[0], NEUTRAL[1], NEUTRAL[2]];
    if channel {
        out.push(CHANNEL);
    }
    out.push(NEUTRAL[3]);
    out
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

/// The layout of a skill menu. `places[k]`: popup row `k`'s line of cells and column; line 0 is the neutral row.
#[derive(Debug, Clone)]
pub(super) struct MenuGrid {
    /// Each row's name and intent (None: the neutral row) and the first line of cells it takes.
    pub(super) lines: Vec<(String, Option<RoundIntent>, i32)>,
    pub(super) places: Vec<(i32, i32)>,
    /// How many lines of cells in all.
    pub(super) depth: i32,
    /// The most cells on a line.
    pub(super) cols: i32,
    /// The round's intent once a pick holds it: the other rows fade.
    pub(super) fixed: Option<RoundIntent>,
    /// The intent rows drawn and clickable. Every listed row keeps its place whether it shows or not, so showing a subset is
    /// this one list and nothing shifts. Every listed intent today.
    pub(super) shown: Vec<RoundIntent>,
    /// The name column and a neutral button's width, measured for the text size (`fit`).
    pub(super) label_w: i32,
    pub(super) button_w: i32,
    /// PK1-S2c: the neutral buttons in their order (the Channel toggle only on a menu that has it), and the Rest
    /// button's text when it names another skill (Feed in a Channel round; measured with the buttons).
    pub(super) buttons: Vec<&'static str>,
    pub(super) rest_name: String,
}

impl MenuGrid {
    /// The layout of `rows` under the rows of `intents` (each with its name; every intent the ruleset lists, so the places
    /// do not depend on `shown`): a row's place is the next free one in its slot, each intent's row on one line.
    pub(super) fn new(intents: &[(RoundIntent, String)], shown: Vec<RoundIntent>, rows: &[Row], fixed: Option<RoundIntent>) -> Self {
        let mut lines = vec![(NEUTRAL_LINE.to_string(), None, 0)];
        let mut places = vec![(0, 0); rows.len()];
        let mut cols = 0;
        let mut depth = 1;
        let buttons = neutral_order(rows.iter().any(|r| r.slot == Some(Slot::Neutral(CHANNEL))));
        for (k, row) in rows.iter().enumerate() {
            if let Some(Slot::Neutral(text)) = row.slot {
                places[k] = (0, buttons.iter().position(|n| *n == text).unwrap_or(0) as i32);
            }
        }
        for (i, name) in intents {
            lines.push((name.clone(), Some(*i), depth));
            let cells: Vec<usize> = (0..rows.len()).filter(|&k| rows[k].slot == Some(Slot::Intent(*i))).collect();
            // INTENT2-U (IR19, Option A): a row stays on one line, however many cells it holds
            for (n, k) in cells.iter().enumerate() {
                places[*k] = (depth, n as i32);
                cols = cols.max(n as i32 + 1);
            }
            depth += 1;
        }
        let rest_name = rows.iter().find(|r| r.slot == Some(Slot::Neutral("Rest"))).map(|r| r.label.clone()).filter(|l| l != "Rest").unwrap_or_default();
        MenuGrid { lines, places, depth, cols, fixed, shown, label_w: 40, button_w: ICON + 7, buttons, rest_name }
    }

    /// Measures the name column and the neutral buttons for the text size.
    pub(super) fn fit(&mut self, p: &mut Painter) {
        let label = self.lines.iter().map(|l| p.text_w(&l.0, BODY)).fold(0.0, f32::max);
        let button = self.buttons.iter().copied().chain(std::iter::once(self.rest_name.as_str())).map(|n| p.text_w(n, SMALL)).fold(0.0, f32::max);
        self.label_w = label.ceil() as i32 + 2;
        self.button_w = (button.ceil() as i32 + 6).max(ICON);
    }

    /// The panel's size with a title row `row_h` high.
    pub(super) fn size(&self, row_h: i32) -> (i32, i32) {
        let neutral = self.buttons.len() as i32 * (self.button_w + GAP);
        let cells = neutral.max(self.cols * (ICON + GAP));
        ((2 * PAD + self.label_w + GAP + cells - GAP).max(MENU_MIN_W), row_h + 2 + self.depth * (ICON + GAP) - GAP + PAD)
    }

    /// Row `k`'s rect in a panel at (`x`, `y`).
    pub(super) fn cell(&self, x: i32, y: i32, row_h: i32, k: usize) -> Rect {
        let (line, col) = self.places.get(k).copied().unwrap_or((0, 0));
        let left = x + PAD + self.label_w + GAP;
        let top = y + row_h + 2 + line * (ICON + GAP);
        if line == 0 {
            Rect::new(left + col * (self.button_w + GAP), top, self.button_w, ICON)
        } else {
            Rect::new(left + col * (ICON + GAP), top, ICON, ICON)
        }
    }

    /// Whether a row of the menu in `slot` is drawn and takes clicks (the neutral row always).
    pub(super) fn shows(&self, slot: Option<Slot>) -> bool {
        !matches!(slot, Some(Slot::Intent(i)) if !self.shown.contains(&i))
    }

    /// Whether `i`'s row fades: another intent fixed the round.
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

/// Draws the menu: panel, title, row names, and each row as a button or an icon, with its widget and hover card.
pub(super) fn draw_menu(ui: &mut Ui<Action>, art: &mut Art, pop: &Popup, grid: &MenuGrid, row_h: i32) {
    let r = pop.rect(row_h);
    ui.p.panel(r, th().game("popup_fill", rgba(14, 20, 34, 252)), th().gold, 3.0);
    // a near miss in a gap or the name column keeps the menu open
    ui.push("popup.panel", "panel", "", r, true, false, Some(Action::MenuGap), Vec::new());
    ui.p.text_fit(r.x as f32 + 5.0, r.y as f32 + 2.0, &pop.title, BODY, r.w as f32 - 10.0, th().gold);
    let line_h = ui.p.line_h(BODY);
    for (name, intent, line) in grid.lines.iter().filter(|l| l.1.is_none_or(|i| grid.shown.contains(&i))) {
        let y = r.y + row_h + 2 + line * (ICON + GAP);
        let ink = intent.map_or(th().dim, intent_ring_color);
        let ink = if grid.faded(*intent) { with_alpha(ink, FADE) } else { ink };
        ui.p.text(r.x as f32 + PAD as f32, y as f32 + (ICON as f32 - line_h) / 2.0, name, BODY, ink);
    }
    for (k, row) in pop.rows.iter().enumerate().filter(|(_, r)| grid.shows(r.slot)) {
        let rr = pop.row(row_h, k);
        let hover = ui.hover(rr);
        let shows = if row.valid { 255 } else { FADE };
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
                let shown = if text == "Rest" && !row.label.is_empty() { row.label.as_str() } else { text };
                ui.p.text_center(rr.x as f32 + rr.w as f32 / 2.0, y, shown, SMALL, with_alpha(th().text, shows.max(110)));
            }
            _ => {
                let edge = if lit { th().gold } else { CELL_EDGE };
                ui.p.panel(rr, with_alpha(th().card, shows), with_alpha(edge, shows), 1.5);
                let px = ((ICON - 2) as f32 * ui.p.v.s).round().max(1.0) as i32;
                let rel = art.first_rel(&row.icon);
                if let Some(img) = rel.as_deref().and_then(|rel| art.image(rel, px, px)) {
                    ui.p.image_alpha(img, Rect::new(rr.x + 1, rr.y + 1, ICON - 2, ICON - 2), shows);
                }
                // the second cell of a skill with both Maneuver kinds: Disengage (its card says so)
                if matches!(&row.act, super::input::RowAct::Pick(p) if matches!(p[..], [rip::Pick::Skill { kind: Some(ManeuverKind::Disengage), .. }])) {
                    ui.p.fill(Rect::new(rr.x + rr.w - 5, rr.y + rr.h - 5, 4, 4), with_alpha(th().text, shows));
                }
            }
        }
        // a pouch item's copies left (INTENT2-U: the corner `xN` badge)
        if let Some(n) = row.count {
            let text = format!("x{n}");
            let (tw, lh) = (ui.p.text_w(&text, SMALL), ui.p.line_h(SMALL));
            text_outlined(&mut ui.p, (rr.x + rr.w) as f32 - tw - 1.0, (rr.y + rr.h) as f32 - lh, &text, SMALL, with_alpha(th().text, shows.max(150)), rgba(0, 0, 0, shows));
        }
        // IF3, IN6: a pick that changes the intent and drops intent-bound buffs: a red corner
        if row.warn {
            ui.p.fill(Rect::new(rr.x + rr.w - 4, rr.y + 1, 3, 3), with_alpha(th().foe, shows));
        }
        ui.push(format!("popup.{k}"), "option", format!("{} {}", row.label, row.detail), rr, row.valid, hover, Some(Action::Choose(k)), row.tip.clone());
    }
}
