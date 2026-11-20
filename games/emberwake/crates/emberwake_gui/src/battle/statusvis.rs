//! STATUSVIS1-B (PR7-57, `docs/specs/STATUSVIS1.md` SV5-SV15, `games/emberwake/docs/specs/SCREEN_CONTRACT_BATTLE.md`
//! 6 and 9.3): the battle screen's pieces that show a status where it acts, drawn from the kit's reads
//! (`emberwake_battle::status_view`): a locked block's grey tile with the status icon (SV15), a number's tint and
//! arrow after its modifiers with its frameless hover (SV6, SV7), the taunt badge over a sprite (SV8) and the riposte
//! chip with no number (SV9). Sizes are in the screen's old units (1 unit = 2 px at 1280 x 720).

use super::plates::icon;
use crate::battle_view::NumLine;
use crate::screens::Cx;
use crate::tokens::{boss_gold, crimson, lock_bar, lock_block};
use aegis_ui::canvas::{rgba, Rect, Rgba};
use aegis_ui::painter::Painter;
use aegis_ui::theme::th;
use aegis_ui::ui::{Tip, Ui, SMALL};
use emberwake_battle::status_view::LockRun;
use emberwake_battle::world::{UnitId, World};

/// SV7: a number's colour: lowered the foe's red (the contract's oxblood light), raised the boss gold, else `plain`.
pub(super) fn num_col(sign: i32, plain: Rgba) -> Rgba {
    match sign.signum() {
        -1 => th().foe,
        1 => boss_gold(),
        _ => plain,
    }
}

/// SV7: the small arrow after a moved number at (`x`, `y`) (its top left, 3 x 4 units): up when raised, down when
/// lowered, in `col`; nothing for 0. Returns the width it took (0 for none).
pub(super) fn arrow(p: &mut Painter, x: f32, y: f32, sign: i32, col: Rgba) -> f32 {
    if sign == 0 {
        return 0.0;
    }
    let (x, y) = (x.round() as i32, y.round() as i32);
    // a 3-wide head and a 1-wide tip on a 1-wide stem: up (tip, head, stem, stem) or down (stem, stem, head, tip)
    let rows: [i32; 4] = if sign > 0 { [1, 3, 1, 1] } else { [1, 1, 3, 1] };
    for (k, w) in rows.iter().enumerate() {
        p.fill(Rect::new(x + (3 - w) / 2, y + k as i32, *w, 1), col);
    }
    4.0
}

/// SV7, SV13: a number's frameless hover: its title, then each line in its tint (a heading in gold).
pub(super) fn num_tip(title: &str, lines: &[NumLine]) -> Tip {
    let mut tip: Tip = vec![(title.to_string(), th().gold)];
    tip.extend(lines.iter().map(|l| (l.text.clone(), if l.sign == 2 { th().gold } else { num_col(l.sign, th().text) })));
    tip
}

/// SV6, SV7: draws `text` at (`x`, `y`) in its tint with the arrow after it, and pushes its frameless hover widget `id`
/// (kind `label`: no frame, only the card). Returns the width taken.
#[allow(clippy::too_many_arguments)]
pub(super) fn number(ui: &mut Ui<super::Action>, id: &str, text: &str, x: f32, y: f32, size: f32, sign: i32, plain: Rgba, tip: Tip) -> f32 {
    let col = num_col(sign, plain);
    ui.p.text(x, y, text, size, col);
    let tw = ui.p.text_w(text, size);
    let aw = arrow(&mut ui.p, x + tw + 1.0, y + 1.5, sign, col);
    let w = tw + if aw > 0.0 { aw + 1.0 } else { 0.0 };
    let h = ui.p.line_h(size).ceil() as i32;
    ui.push(id, "label", text, Rect::new(x as i32, y as i32, w.ceil() as i32 + 1, h), true, false, None, tip);
    w
}

/// SV15: a locked block's tile in `r` (one tile over the whole run): the grey fill and rim, the status icon in the
/// skill-icon frame at its centre and a grey bar under it, as the strip's other tiles are built.
pub(super) fn lock_tile(ui: &mut Ui<super::Action>, cx: &mut Cx, r: Rect, status: &str) {
    let grey = lock_block();
    ui.p.panel(r, grey, lock_bar(), 0.0);
    let side = super::tips::BLOCK_ICON.min(r.h - 2).min(r.w - 2).max(2);
    let sq = Rect::new(r.x + (r.w - side) / 2, r.y + (r.h - side) / 2 - 1, side, side);
    ui.p.fill(sq, rgba(10, 12, 16, 235));
    if !icon(ui, cx, &emberwake_content::art::status_icon(status), sq.inset(1), 255) {
        let letter: String = emberwake_rules::status::display_name(status).chars().take(1).collect();
        ui.p.text_center(sq.x as f32 + sq.w as f32 / 2.0, sq.y as f32 + 2.0, &letter, SMALL, lock_bar());
    }
    ui.p.panel(sq, rgba(0, 0, 0, 0), lock_bar(), 0.0);
    ui.p.fill(Rect::new(r.x + 2, r.y + r.h - 2, r.w - 4, 1), lock_bar());
}

/// SV5, SV15: the hover lines of lock run `run`: the status, the blocks it takes, no plan there.
pub(super) fn lock_tip(run: &LockRun) -> Tip {
    let name = emberwake_rules::status::display_name(&run.status);
    let blocks = if run.start == run.end { format!("Block {} locked", run.start) } else { format!("Blocks {}-{} locked", run.start, run.end) };
    vec![(name, th().foe), (blocks, th().text), ("No move can be planned here.".into(), th().dim)]
}

/// SV8: the taunt badge's side (20 px) and its pointer.
pub(super) const BADGE: i32 = 10;

/// SV8: the taunt badge over unit `u`'s sprite, its pointer's tip at (`cx0`, `top`) (the sprite's top centre): a dark
/// slate square with a crimson rim and the taunt status's icon, a pointer down to the sprite, bobbing 1 px (`bob`); its
/// widget `board.taunt.<u>` names the unit foes must target and the rounds left.
pub(super) fn taunt_badge(ui: &mut Ui<super::Action>, cx: &mut Cx, w: &World, u: UnitId, status: &str, (cx0, top): (i32, i32), bob: i32) {
    let r = Rect::new(cx0 - BADGE / 2, top - BADGE - 3 + bob, BADGE, BADGE);
    let rim = crimson();
    ui.p.fill(r, th().game("chip", rgba(30, 44, 70, 255)));
    ui.p.fill(Rect::new(cx0 - 1, r.y + r.h, 3, 1), rim);
    ui.p.fill(Rect::new(cx0, r.y + r.h + 1, 1, 1), rim);
    if !icon(ui, cx, &emberwake_content::art::status_icon(status), r.inset(1), 255) {
        ui.p.text_center(cx0 as f32, r.y as f32 + 1.0, "!", SMALL, rim);
    }
    ui.p.panel(r, rgba(0, 0, 0, 0), rim, 0.0);
    let unit = &w.units[u];
    let left = unit.st.rounds_left.get(status).copied().filter(|n| *n > 0);
    let rounds = match left {
        Some(1) => "1 round left".to_string(),
        Some(n) => format!("{n} rounds left"),
        None => "Lasts".to_string(),
    };
    let tip = vec![(emberwake_rules::status::display_name(status), rim), (format!("Foes must target {}", unit.name), th().text), (rounds, th().dim)];
    ui.push(format!("board.taunt.{u}"), "badge", format!("{} taunts", unit.name), r, true, false, None, tip);
}

/// SV8: the status that gives `u` the taunt badge (its first active `targeting` pull status), if any.
pub(super) fn taunt_of(w: &World, u: UnitId) -> Option<String> {
    w.units.get(u)?.st.active.iter().find(|s| emberwake_rules::status::has_targeting_pull(s)).cloned()
}

/// SV8: the badge's bob this frame: 0 or -1 unit, turn about every 600 ms of `now_ms`; 0 when still.
pub(super) fn bob(now_ms: u64, still: bool) -> i32 {
    if still { 0 } else { -(((now_ms / 600) % 2) as i32) }
}

/// SV9: the riposte chip's width.
pub(super) fn riposte_chip_w(ui: &mut Ui<super::Action>) -> i32 {
    ui.p.text_w("Riposte", SMALL).ceil() as i32 + 4
}

/// SV9: the riposte chip at (`x`, `y`): "Riposte" with no number, crimson rim; its hover the likely return on each
/// hitter (`returns`: name and HP). Returns its width.
pub(super) fn riposte_chip(ui: &mut Ui<super::Action>, id: &str, x: i32, y: i32, returns: &[(String, i32)]) -> i32 {
    let text = "Riposte";
    let w = riposte_chip_w(ui);
    let r = Rect::new(x, y, w, 8);
    ui.p.panel(r, rgba(10, 12, 16, 235), crimson(), 0.0);
    ui.p.text(x as f32 + 2.0, y as f32 + 0.5, text, SMALL, th().text);
    let mut tip: Tip = vec![("Riposte".into(), crimson()), ("A melee hit on it draws a counter.".into(), th().text)];
    tip.extend(returns.iter().map(|(n, hp)| (format!("Likely return on {n}: about {hp} HP"), th().foe)));
    ui.push(id, "badge", text, r, true, false, None, tip);
    w
}

/// SV14, SV15 (take 36): a row's one target in the full timeline view's aside square `sq`: a small arrow, then the
/// target's face in a crimson frame (an enemy) or a teal frame with the arrow teal (an ally or itself), a small amber
/// ring on the face's corner for an area skill's main target, a 3 x 3 floor patch with the target tile lit for a ground
/// tile, a dim dash for none; its hover (`timeline.target.<track>`) names it.
#[allow(clippy::too_many_arguments)]
pub(super) fn row_target(ui: &mut Ui<super::Action>, cx: &mut Cx, small: &mut crate::portraits::Faces, looks: &super::plates::Looks, w: &World, key: &str, aim: emberwake_battle::status_view::RowAim, sq: Rect) {
    use emberwake_battle::status_view::RowTarget;
    let side = sq.h - 4;
    let face = Rect::new(sq.x + sq.w - side - 1, sq.y + 2, side, side);
    let teal = crate::tokens::guest();
    let (arrow_col, frame, unit) = match aim.target {
        RowTarget::None => {
            ui.p.fill(Rect::new(face.x + side / 4, face.y + side / 2, side / 2, 1), th().dim);
            ui.push(format!("timeline.target.{key}"), "label", "No target", sq, true, false, None, vec![("No target this round".into(), th().dim), ("A Maneuver or Rest only.".into(), th().text)]);
            return;
        }
        RowTarget::Foe(t) => (th().dim, crimson(), Some(t)),
        RowTarget::Ally(t) => (teal, teal, Some(t)),
        RowTarget::Myself => (teal, teal, key.strip_prefix('u').and_then(|n| n.parse().ok())),
        RowTarget::Ground(_) => (th().dim, crate::tokens::planned(), None),
    };
    // the arrow: a stem and a head pointing at the face
    let ay = face.y + side / 2;
    ui.p.fill(Rect::new(sq.x + 1, ay, 2, 1), arrow_col);
    ui.p.fill(Rect::new(sq.x + 2, ay - 1, 1, 3), arrow_col);
    let mut tip: Tip = Vec::new();
    match (aim.target, unit) {
        (RowTarget::Ground(c), _) => {
            // a 3 x 3 floor patch, the target tile lit
            let cell = ((side - 2) / 3).max(1);
            let x0 = face.x + (side - 3 * cell) / 2;
            let y0 = face.y + (side - 3 * cell) / 2;
            for gy in 0..3 {
                for gx in 0..3 {
                    let r = Rect::new(x0 + gx * cell, y0 + gy * cell, cell - 1, cell - 1);
                    ui.p.fill(r, if (gx, gy) == (1, 1) { crate::tokens::planned() } else { th().game("battle_track", rgba(30, 40, 60, 255)) });
                }
            }
            tip.push(("Target: a ground tile".into(), crate::tokens::planned()));
            tip.push((format!("Tile ({}, {})", c.x, c.y), th().text));
        }
        (_, Some(t)) => {
            if let Some(l) = looks.get(t) {
                small.draw(&mut ui.p, cx, &l.0, &l.1, super::TIMELINE_SURFACE, face.inset(1));
            }
            ui.p.panel(face, rgba(0, 0, 0, 0), frame, 0.0);
            if aim.area {
                let rr = (side / 4).max(2) as f32;
                ui.p.ellipse_ring(face.x as f32 + face.w as f32 - rr, face.y as f32 + rr, rr, rr, crate::tokens::planned());
            }
            let name = w.units.get(t).map_or_else(String::new, |u| u.name.clone());
            let whom = match aim.target {
                RowTarget::Myself => "Target: itself".to_string(),
                RowTarget::Ally(_) => format!("Target: {name} (ally)"),
                _ => format!("Target: {name}"),
            };
            tip.push((whom, frame));
            if aim.area {
                tip.push(("An area skill's main target".into(), crate::tokens::planned()));
            }
        }
        _ => {}
    }
    let text = tip.first().map(|t| t.0.clone()).unwrap_or_default();
    ui.push(format!("timeline.target.{key}"), "label", text, sq, true, false, None, tip);
}

/// SV3, SV6, SV13 (PR7-57): a plate's info widgets (its planned-loss number, status tabs and "+N", passive sockets:
/// `<plate>.incoming`, `.status.`, `.passive.`, `.socket.`) pushed since `from` move after the plate's own card widget,
/// so their hovers show over the plate's card (they take no click: the plate keeps its clicks).
pub(super) fn lift_plate_info(ui: &mut Ui<super::Action>, from: usize, plate: &str) {
    let parts = [".incoming", ".status.", ".passive.", ".socket."];
    let info = |id: &str| id.strip_prefix(plate).is_some_and(|rest| parts.iter().any(|p| rest.starts_with(p)));
    let tail = ui.widgets.split_off(from.min(ui.widgets.len()));
    let (lifted, kept): (Vec<_>, Vec<_>) = tail.into_iter().partition(|w| info(&w.id));
    ui.widgets.extend(kept);
    ui.widgets.extend(lifted);
}
