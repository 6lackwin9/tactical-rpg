//! OBJUI1 (`PR7-31`; `OBJ1-U` with `MORALE1-U`, `docs/archive/specs/emberwake/OBJ1.md` and `MORALE1.md` section 3):
//! the objective and morale marks on the board's tokens, drawn from the kit's reads (`emberwake_battle::obj_view`).
//! Like the status tabs (`SCREEN_CONTRACT_BATTLE` section 6) a chip is a small glyph with no number; its words are on
//! its hover. A column at the token's lower left, bottom up: the guest's shield (gold rim when it must survive), the
//! morale state (three bars lit by state, gold when fearless; the group letter instead when the battle has several
//! groups), the leader's pennant, the slay target's crown. No Enraged chip: an enraged foe shows by its amber tint and
//! raised numbers (ENRAGE2 EN5, BOSSUI1 BU4). Sizes are in the screen's old units (1 unit = 2 px at 1280 x 720).

use super::plates::crown;
use crate::screens::Cx;
use crate::tokens::{boss_gold, guest, planned};
use aegis_ui::canvas::{rgba, Rect, Rgba};
use aegis_ui::theme::th;
use aegis_ui::ui::{Tip, Ui, SMALL};
use emberwake_battle::obj_view::{self as ov, MoraleMark};
use emberwake_battle::session::Session;
use emberwake_battle::world::UnitId;

/// A chip's side.
pub(super) const CHIP: i32 = 8;

/// A morale state's colour: steady parchment, shaken amber, wavering the foe's red, fearless gold.
pub(super) fn state_col(state: &str) -> Rgba {
    match state {
        "shaken" => planned(),
        "wavering" | "broken" => th().foe,
        "fearless" => boss_gold(),
        _ => th().text,
    }
}

/// The bars a state lights (of 3).
fn bars_of(state: &str) -> i32 {
    match state {
        "steady" | "fearless" => 3,
        "shaken" => 2,
        _ => 1,
    }
}

/// A morale mark's hover: the title, the flavour line, the card's line on kills and crits.
pub(super) fn morale_tip(m: &MoraleMark) -> Tip {
    vec![(m.title(), state_col(m.state)), (m.line.to_string(), th().text), (ov::INSPECT_LINE.to_string(), th().dim)]
}

/// The chip back: a dark square with a rim.
fn back(ui: &mut Ui<super::Action>, r: Rect, rim: Rgba) {
    ui.p.fill(r, th().game("chip", rgba(30, 44, 70, 255)));
    ui.p.panel(r, rgba(0, 0, 0, 0), rim, 0.0);
}

/// The objective and morale chips of unit `i`, bottom up from the lower left corner (`x`, `bottom`).
pub(super) fn token_chips(ui: &mut Ui<super::Action>, cx: &mut Cx, s: &Session, i: UnitId, (x, bottom): (i32, i32)) {
    let marks = ov::marks(s, i);
    let name = s.w.units[i].name.clone();
    let mut y = bottom - CHIP;
    let next = |y: &mut i32| {
        let r = Rect::new(x, *y, CHIP, CHIP);
        *y -= CHIP + 1;
        r
    };
    if marks.guest {
        let r = next(&mut y);
        back(ui, r, if marks.must_survive { boss_gold() } else { guest() });
        shield(ui, r.inset(2), guest());
        let mut tip: Tip = vec![("Guest".into(), guest()), (format!("{name} fights beside the party on its own."), th().text)];
        if marks.must_survive {
            tip.push(("Must survive".into(), boss_gold()));
        }
        ui.push(format!("board.guest.{i}"), "chip", if marks.must_survive { "Guest, must survive".to_string() } else { "Guest".to_string() }, r, true, false, None, tip);
    }
    if let Some(m) = ov::morale_mark(s, i) {
        morale_chip(ui, &m, next(&mut y), &format!("board.morale.{i}"));
    }
    if ov::is_leader(s, i) {
        leader_chip(ui, next(&mut y), &format!("board.leader.{i}"));
    }
    if marks.slay {
        let r = next(&mut y);
        back(ui, r, boss_gold());
        crown(ui, cx, r.inset(1));
        ui.push(format!("board.slay.{i}"), "chip", "Slay target".to_string(), r, true, false, None, vec![("Slay target".into(), boss_gold()), (format!("Defeat {name} to win."), th().text)]);
    }
}

/// A morale chip in `r`: three bars lit by state (the group letter instead with several groups); widget `id`.
pub(super) fn morale_chip(ui: &mut Ui<super::Action>, m: &MoraleMark, r: Rect, id: &str) {
    let col = state_col(m.state);
    back(ui, r, col);
    match m.group {
        Some(g) => ui.p.text_center(r.x as f32 + r.w as f32 / 2.0, r.y as f32 + (r.h as f32 - ui.p.line_h(SMALL)) / 2.0, &g.to_string(), SMALL, col),
        None => {
            let lit = bars_of(m.state);
            for k in 0..3 {
                let h = 2 + k;
                ui.p.fill(Rect::new(r.x + 1 + k * 2, r.y + r.h - 1 - h, 1, h), if k < lit { col } else { th().edge });
            }
        }
    }
    ui.push(id, "chip", m.title(), r, true, false, None, morale_tip(m));
}

/// The leader's chip in `r` (a gold pennant); widget `id`.
pub(super) fn leader_chip(ui: &mut Ui<super::Action>, r: Rect, id: &str) {
    back(ui, r, boss_gold());
    pennant(ui, r.inset(1), boss_gold());
    ui.push(id, "chip", "Leader".to_string(), r, true, false, None, vec![("Leader".into(), boss_gold()), (ov::LEADER_LINE.into(), th().text)]);
}

/// The board's chip widgets moved after the units' (the Stage view pushes its unit widgets last, over the chips):
/// a hover on a chip reaches the chip.
pub(super) fn raise_chips(ui: &mut Ui<super::Action>) {
    let is_chip = |w: &aegis_ui::ui::Widget<super::Action>| w.kind == "chip" && w.id.starts_with("board.");
    let (chips, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut ui.widgets).into_iter().partition(is_chip);
    ui.widgets = rest;
    ui.widgets.extend(chips);
}

/// A small heater shield in `r`.
fn shield(ui: &mut Ui<super::Action>, r: Rect, col: Rgba) {
    let top = r.h / 2;
    ui.p.fill(Rect::new(r.x, r.y, r.w, top.max(1)), col);
    for k in 0..(r.h - top) {
        let inset = (k + 1) * r.w / (2 * (r.h - top + 1));
        ui.p.fill(Rect::new(r.x + inset, r.y + top + k, (r.w - 2 * inset).max(1), 1), col);
    }
}

/// A small pennant on a pole in `r`.
fn pennant(ui: &mut Ui<super::Action>, r: Rect, col: Rgba) {
    ui.p.fill(Rect::new(r.x + 1, r.y, 1, r.h), col);
    for k in 0..(r.h / 2) {
        ui.p.fill(Rect::new(r.x + 2, r.y + k, (r.w - 2 - k).max(1), 1), col);
    }
}
