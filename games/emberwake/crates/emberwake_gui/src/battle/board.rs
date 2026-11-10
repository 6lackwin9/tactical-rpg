//! The battle board: the Flat and Stage views' passes over the tiles, tokens, markers and hit rects (`Board`, `draw_flat`, `draw_stage`), status loops on tokens, board geometry helpers.

use aegis_ui::theme::th;
use super::marks::{bars_shown, break_plate, broken_units, draw_arrows, draw_hold_tiles, draw_hook_marks, draw_pushes, exit_sign, poise_bar, poise_shown, HoldMarks, HookMarks};
use super::overlays::{draw_overlay, Layers, Overlay};
use super::{Action, ALLY, AMBER, DASH_MS, GRID_X, GRID_Y, TILE};
use super::tips::{intent_glyph, intent_ring_color, ring_intent, unit_tip};
use super::tokens::{body_keys, hit_flash, token_frame, token_outline, Float, Tokens, UnitAnim};
use aegis_ui::canvas::{rgb, rgba, Rect, Rgba};
use crate::fx;
use aegis_ui::fx::{Fx, Layer};
use crate::stage::{self, Geom, Item, Pass, Through, View};
use crate::tuning::Tuning;
use crate::screens::Cx;
use emberwake_battle::intent::Intent;
use aegis_ui::painter::Painter;
use aegis_ui::ui::{Tip, Ui, BODY, RADIUS, SMALL, TITLE};
use emberwake_battle::command::CmdKind;
use emberwake_battle::grid::{V2, GRID_H, GRID_W};
use emberwake_battle::session::Session;
use aegis_core::assets::{ClipFrame, Facing};
use aegis_ui::art::Art;
use emberwake_battle::world::World;
use aegis_core::clock::Instant;

/// A cell's foot point as an effect point (4.1: an area's aim, support motes, falls).
pub(super) fn foot_point(geo: &Geom, c: V2) -> (f32, f32) {
    let (x, y) = geo.foot(c);
    (x as f32, y as f32)
}

pub(super) fn foot_points(geo: &Geom, cells: &[V2]) -> Vec<(f32, f32)> {
    cells.iter().map(|&c| foot_point(geo, c)).collect()
}

/// FX1 4.1: an action's effect places in the view `geo` (the anchors of its cells; the Stage's squash 0.75, Flat's 1).
/// FX1-L2: an area's aim is its cell's foot point (4.1); a target further above or below than aside faces the actor up
/// or down (DIR4-G `Facing::toward`), so its swing turns toward it.
pub(super) fn place(geo: &Geom, from: V2, to: V2, area: bool) -> fx::Place {
    let squash = if geo.view == View::Stage { 0.75 } else { 1.0 };
    let to_pt = if area { foot_point(geo, to) } else { fx::anchor(geo, to) };
    let vertical = (to.y - from.y).abs() > (to.x - from.x).abs();
    fx::Place { from: fx::anchor(geo, from), to: to_pt, squash, vertical }
}

pub(super) fn hp_bar(p: &mut Painter, r: Rect, hp: i32, max: i32, color: Rgba) {
    p.fill(r, rgb(46, 24, 30));
    let w = if max > 0 { (r.w * hp.max(0) + max - 1) / max } else { 0 };
    p.fill(Rect::new(r.x, r.y, w.min(r.w), r.h), color);
}

pub(super) fn cell_rect(c: V2) -> Rect {
    Rect::new(GRID_X + c.x * TILE, GRID_Y + c.y * TILE, TILE, TILE)
}

/// What the board draws from the battle screen in one frame (borrowed apart from the session).
pub(super) struct Board<'a> {
    pub(super) s: &'a Session,
    pub(super) tokens: &'a mut Tokens,
    pub(super) floats: &'a [Float],
    pub(super) flashes: &'a [(usize, Instant)],
    pub(super) popup: bool,
    /// The FX clock (ms; FX1-L1).
    pub(super) anim_ms: f64,
    pub(super) sel: Option<usize>,
    /// ENCOUNTER2-C DT8: the open talk's speaker on the field (wears the selection ring; None without one).
    pub(super) talker: Option<usize>,
    pub(super) planning: bool,
    /// INTENT1: the enemy tells while planning; drawn while `show_tells` (the Show enemy tells setting, I).
    pub(super) tells: &'a [Intent],
    pub(super) show_tells: bool,
    /// STAGE1 3.9: the unit whose party or hostile card the mouse is over.
    pub(super) card_hover: Option<usize>,
    /// INPUT1 IS1, AR3 (decision "INPUT1 answers, part 4" Q15): the inspected unit; its target arrow shows by default.
    pub(super) inspect: Option<usize>,
    /// VISCLAR1 VC2: the focus set but the hovered unit (each view adds the unit under the mouse).
    pub(super) focus: &'a [usize],
    /// FX1-L1 slice 2: the live effects (drawn at `anim_ms` on the ground and air layers, 4.1) and the camera offset
    /// the board's passes are shifted by.
    pub(super) fx: &'a Fx,
    pub(super) shift: (i32, i32),
    /// FX1-L2: the miss sidesteps (4.3).
    pub(super) sidesteps: &'a [fx::Sidestep],
    /// RULESET1-U slice 3: the round intent's ground overlays (None in classic: the board draws as before).
    pub(super) overlay: Option<&'a Overlay>,
    /// RULESET1-U slice 4 (UI5): every unit's round intent record (the board's intent glyphs; empty in classic).
    pub(super) rounds: &'a [Intent],
    /// RULESET1-U slice 5: the overlay groups on (`Layers`) and Alt held (VISCLAR1: every bar and target arrow shows).
    pub(super) layers: Layers,
    pub(super) alt: bool,
    /// HOOKS1-UI slice 1: the ground hazards and the arena telegraph (empty in classic).
    pub(super) marks: &'a HookMarks,
    /// HOOKS1-UI slice 2: each unit's class name for the hover cards (`class_names`).
    pub(super) classes: &'a [String],
    /// HOOKS1-UI slice 4: the selected hero's push and pull forecasts (planning, layer `pushes`).
    pub(super) pushes: &'a [emberwake_battle::forced::Forecast],
    /// ENCOUNTER2-U: a hold escape's tiles and gate (None without one).
    pub(super) hold: Option<&'a HoldMarks>,
}

/// INPUT1 MR12: `unit` moved in this step by a dash: a command of its whose start is this block made the dash board event
/// (`MoveKind::Dash`, the move of a skill with a position).
pub(super) fn dashed(s: &Session, unit: usize) -> bool {
    s.resolver.scheduled.iter().any(|a| a.unit == unit && a.start == s.current_block && a.command.moved == Some(emberwake_battle::command::MoveKind::Dash))
}

/// An action clip a token wants starts with the first candidate its body has (SPRITE1-U), once the clock reaches its
/// start (FX1-L1: after the actor's slide, so the walk shows until then); WINDUP1 W4: after a held wind-up from the
/// clip's `strike` frame.
pub(super) fn start_wanted(tokens: &mut Tokens, art: &mut Art, i: usize, body: Option<&str>, now: u64) {
    if let (Some(a), Some(b)) = (tokens.anims.get_mut(i), body) {
        // INPUT1 MR12: a dash plays the rig's `dash` clip over its slide when the body has one (else its walk shows)
        if std::mem::take(&mut a.dash_clip) && art.has_clip(b, "dash") {
            if let Some((_, t0)) = a.slide {
                a.player.play_for("dash", t0, DASH_MS);
            }
        }
        if a.want.as_ref().is_some_and(|w| now >= w.1) {
            let (cands, t0) = a.want.take().unwrap_or_default();
            if let Some(c) = cands.into_iter().find(|c| art.has_clip(b, c)) {
                // WINDUP1 W4: from its `strike` frame after a held wind-up (0 when the row marks none)
                let from = if a.strike { art.clips(b).and_then(|k| k.clip(c)).map_or(0, |k| k.strike) } else { 0 };
                a.player.play_from(c, t0, from);
            }
            a.strike = false;
        }
    }
}

/// WINDUP1 W9 (WQ5): the Tallow Amber outline of a unit showing a wind-up (resolution only; `windup_outline_alpha`, 0
/// off); the selected and hovered outlines win (the callers ask this last).
pub(super) fn windup_outline(a: &UnitAnim, t: &Tuning) -> Option<Rgba> {
    (a.windup.is_some() && t.windup_outline_alpha > 0).then(|| with_alpha(AMBER, t.windup_outline_alpha))
}

/// WINDUP1 6.2: the clip each token shows at `now` (the side clip of the row its player draws, a missing clip's
/// fallback included; "" without a body), for the agent link's `tokens`.
pub(super) fn note_drawn(tokens: &mut Tokens, s: &Session, art: &mut Art, now: u64) {
    for (i, a) in tokens.anims.iter_mut().enumerate().take(s.w.units.len()) {
        let body = art.first_body(&body_keys(&s.art[i], &s.w.units[i]));
        a.drawn = body.and_then(|b| art.clips(&b).and_then(|c| a.player.frame(c, now)).map(|f| f.clip)).unwrap_or_default();
    }
}

/// FX1-L1: a struck unit's flash (wall ms since its impact) and its token shake (4.5; none under reduce motion and skip).
fn flash_of(flashes: &[(usize, Instant)], i: usize, now: Instant, t: &Tuning, still: bool) -> (bool, i32) {
    match flashes.iter().rev().find(|f| f.0 == i) {
        Some(f) => (true, fx::token_shake(t.hit_shake_px, now.saturating_duration_since(f.1).as_millis() as u64, t.hit_flash_ms, still)),
        None => (false, 0),
    }
}

/// A fallen unit still playing its death clip (and holding its last frame for `death_hold_ms`); after it (or with no
/// clip) the dead marker shows.
fn still_dying(art: &mut Art, a: &UnitAnim, body: Option<&str>, now: u64, instant: bool, hold_ms: u64) -> bool {
    match (body, a.fell) {
        (Some(b), Some(_)) if !instant && art.has_clip(b, "death") => {
            art.clips(b).is_some_and(|clips| a.player.playing(clips, now) == "death" && a.player.held_for(clips, now).is_none_or(|h| h < hold_ms))
        }
        _ => false,
    }
}

/// STATUSFX1: one token's status looks this frame (`fx::status_frame`) at its drawn foot point (the slide; Flat: the
/// token's centre), facing left or not.
pub(super) struct StatusOn {
    pub(super) unit: usize,
    pub(super) foot: (f32, f32),
    pub(super) left: bool,
    pub(super) looks: Vec<fx::Drawn>,
}

/// STATUSFX1 S1, S2: the looks unit `i`'s token draws this frame (none once it has fallen, S1), noted for the agent
/// link's `status_fx` (S9).
fn status_draws(tokens: &mut Tokens, i: usize, alive: bool, wall: u64, now: u64, t: &Tuning) -> Vec<fx::Drawn> {
    let Some(a) = tokens.anims.get_mut(i) else { return Vec::new() };
    let looks = if alive { fx::status_frame(&a.marks, &mut a.slots, wall, now, t.status_cycle_ms) } else { Vec::new() };
    a.status_fx = looks.iter().map(|d| d.look.id).collect();
    looks
}

/// STATUSFX1, the Flat view: the standing units' status looks at their drawn centres (with the slide).
fn flat_status(tokens: &mut Tokens, w: &World, geo: &Geom, t: &Tuning, anim_now: u64, now: Instant, instant: bool) -> Vec<StatusOn> {
    let wall = tokens.wall_ms(now);
    for a in &mut tokens.anims {
        a.status_fx.clear();
    }
    let mut out = Vec::new();
    for i in 0..w.units.len() {
        let Some(pos) = w.grid.pos(i) else { continue };
        let (_, alive) = tokens.shown(i, &w.units[i], w.units.len());
        let (mut dx, mut dy, mut left) = (0, 0, !w.allies.contains(&i));
        if let Some(a) = tokens.anims.get(i) {
            left = a.faces(anim_now, if instant { 0 } else { t.walk_ms }).left;
            let ms = a.slide_ms(t.walk_ms);
            if let Some((from, t0)) = a.slide.filter(|s| anim_now.saturating_sub(s.1) < ms && !instant) {
                (dx, dy) = geo.slide(from, pos, 1.0 - anim_now.saturating_sub(t0) as f32 / ms.max(1) as f32);
            }
        }
        let looks = status_draws(tokens, i, alive, wall, anim_now, t);
        if alive {
            let (x, y) = geo.foot(pos);
            out.push(StatusOn { unit: i, foot: ((x + dx) as f32, (y + dy) as f32), left, looks });
        }
    }
    out
}

/// STATUSFX1 S6, S7: the status loops on `layer` (feet looks on the ground, head and body looks in the air) at clock
/// `now`, or still (`Looped::draw_still`) under reduce motion.
pub(super) fn draw_status_loops(p: &mut Painter, on: &[StatusOn], layer: Layer, geo: &Geom, t: &Tuning, now: u64, still: bool) {
    for s in on {
        for d in &s.looks {
            let Some(l) = fx::status_loop(d.look, geo, t, s.foot, s.left, d.start, fx::status_seed(s.unit, d.look.id), d.pending) else { continue };
            if l.layer() != layer {
                continue;
            }
            if still {
                l.draw_still(p);
            } else {
                l.draw(p, now);
            }
        }
    }
}

/// STATUSFX1: a tint, the silhouette of the token frame `f` of (body, palette) in `col` at alpha `a`, mirrored with it.
fn status_tint(p: &mut Painter, art: &mut Art, (body, palette, f): (&str, &str, &ClipFrame), tr: Rect, facing: Facing, col: Rgba, a: u8) {
    if a == 0 {
        return;
    }
    if let Some(img) = art.body_silhouette(body, palette, &f.clip, f.frame, tr.w, col, facing.mirrors(f)) {
        p.image_alpha(img, tr, a);
    }
}

/// `a` times `k` / 255 (an alpha under the token's).
fn scaled(a: u8, k: u8) -> u8 {
    ((a as u32 * k as u32 + 127) / 255) as u8
}

/// STAGE1 4.1, the Flat view (the board as it was before STAGE1, until the user's review): 32 x 32 cells at (128, 30),
/// the `common` tiles, tree tiles flat, tokens `flat_token_px` on a filled foot ellipse at `foot_alpha`, arrows centre
/// to centre over the units, the cell widgets.
/// The Flat board; returns the unit whose tile the mouse is over (VC2's hovered unit).
pub(super) fn draw_flat(b: Board, ui: &mut Ui<Action>, cx: &mut Cx, tuning: &Tuning, now: Instant) -> Option<usize> {
    let Board { s, tokens, floats, flashes, popup, anim_ms, sel, talker, planning, tells, show_tells, card_hover, inspect, focus, fx: live, shift, sidesteps, overlay, rounds, layers, alt, marks, classes, pushes, hold } = b;
    let w = &s.w;
    // VISCLAR1 VC2: the unit under the mouse (its tile) joins the focus set; AR3: the arrows of the selected and the
    // inspected unit (Alt: all)
    let tile_hover = if popup { None } else { (0..w.units.len()).find(|&u| w.units[u].is_active() && w.grid.pos(u).is_some_and(|p| ui.hover(cell_rect(p)))) };
    let mut focus = focus.to_vec();
    focus.extend(tile_hover.or(card_hover));
    let shown = overlay.map(|o| o.shown(&layers, sel, inspect, alt));
    let panel = Rect::new(GRID_X - 2, GRID_Y - 2, GRID_W * TILE + 4, GRID_H * TILE + 4);
    ui.p.panel(panel, th().game("board_tray", rgb(8, 10, 16)), th().edge, RADIUS);
    // FX1 4.1: the board shifted by the camera, kept inside its panel while it shakes (at rest the Flat board draws
    // unclipped, as before)
    if shift != (0, 0) {
        ui.p.clip(Some(panel));
    }
    ui.p.shift(shift.0, shift.1);
    for gy in 0..GRID_H {
        for gx in 0..GRID_W {
            let c = V2::new(gx, gy);
            let r = cell_rect(c);
            let tid = w.grid.terrain_id(c);
            match cx.art.tile_in(&emberwake_content::art::BATTLE_SETS, tid, gx, gy, TILE) {
                Some(img) => ui.p.image(img, r),
                None => ui.p.fill(r, stage::terrain_color(tid)),
            }
            if w.grid.is_exit(c) {
                // ENCOUNTER2 H7: a hold escape's exit reads SHUT or OPEN (layer `gate`)
                let (word, fill, edge) = exit_sign(hold);
                ui.p.panel(r.inset(7), fill, edge, 2.0);
                ui.p.text_center(r.x as f32 + 16.0, r.y as f32 + 11.0, word, SMALL, th().bg);
            }
            if planning && sel.is_some() && ui.hover(r) && !popup {
                ui.p.panel(r.inset(1), rgba(255, 255, 255, 30), rgba(255, 255, 255, 160), 1.5);
            }
            let mut tip: Tip = if w.grid.is_exit(c) { vec![("Exit".into(), th().good), ("A unit that steps here leaves the battle.".into(), th().text)] } else { Vec::new() };
            tip.extend(marks.tile_tip(c));
            tip.extend(hold.map(|h| h.tile_tip(w, c)).unwrap_or_default());
            ui.push(format!("tile.{gx}.{gy}"), "tile", tid, r, planning, false, if w.occupant(c).is_none() { Some(Action::TileClick(c)) } else { None }, tip);
        }
    }
    // HOOKS1-UI slice 1: the ground hazards and the arena's struck cells, under the round overlays
    draw_hook_marks(&mut ui.p, marks, &Geom::new(View::Flat, tuning));
    if let Some(h) = hold {
        draw_hold_tiles(&mut ui.p, h, &Geom::new(View::Flat, tuning));
    }
    let center = |c: V2| ((GRID_X + c.x * TILE + TILE / 2) as f32, (GRID_Y + c.y * TILE + TILE / 2) as f32);
    if let Some(o) = &shown {
        // RULESET1-U slice 3: the round intent's overlays on the ground; VISCLAR1 AR5: the target arrows over them
        // (under the tokens)
        draw_overlay(&mut ui.p, o, &Geom::new(View::Flat, tuning));
        draw_arrows(&mut ui.p, &o.arrows, &Geom::new(View::Flat, tuning));
    } else if let Some(h) = sel {
        if w.units[h].is_active() && planning {
            let st = s.plan.compile(w, h);
            let mut from = w.pos(h);
            for e in &st.entries {
                if e.pos_after != from && e.pos_after.x >= 0 {
                    let (a, b) = (center(from), center(e.pos_after));
                    ui.p.line(a.0, a.1, b.0, b.1, th().gold, 3.0);
                    ui.p.panel(cell_rect(e.pos_after).inset(3), rgba(244, 198, 84, 40), th().gold, 2.0);
                    from = e.pos_after;
                }
            }
        }
    }
    // classic: the tuning's enemy attack lines (VISCLAR1 AR5: the round intent's target arrows replace them)
    if tuning.intent_arrows && planning && overlay.is_none() {
        let mut seen = Vec::new();
        for a in &s.resolver.scheduled {
            if !w.enemies.contains(&a.unit) || seen.contains(&a.unit) {
                continue;
            }
            if let CmdKind::Attack { target: Some(t), .. } = a.command.kind {
                seen.push(a.unit);
                let (p, q) = (center(w.pos(a.unit)), center(w.pos(t)));
                ui.p.line(p.0, p.1, q.0, q.1, rgba(240, 100, 92, 210), 4.0);
            }
        }
    }
    // FX1 4.1: the ground layer under the units
    let anim_now = anim_ms as u64;
    let instant = cx.settings.v["skip_anim"].as_bool().unwrap_or(false);
    let reduce = cx.settings.reduce_motion();
    let still = UnitAnim::default();
    let geo = Geom::new(View::Flat, tuning);
    // STATUSFX1: the standing units' status looks (feet on the ground layer, head and body in the air), the status
    // loops before FX1's live effects (S6)
    let status = flat_status(tokens, w, &geo, tuning, anim_now, now, instant);
    draw_status_loops(&mut ui.p, &status, Layer::Ground, &geo, tuning, anim_now, reduce);
    live.draw(&mut ui.p, anim_now, Layer::Ground);
    // SPRITE1-U: each unit stands on a flat ellipse in its side's colour and plays its clips (anims) in its facing
    // (DIR4-G: toward its target or its step, `Tokens::catch_up`, `UnitAnim::faces`; `token_frame`, its outline with it);
    // the selected hero's sprite has a gold outline, a hovered unit's a faint white one. A unit without a body keeps
    // the letter. STATUSFX1: a tint right after the sprite (under the hit flash); a stealthed token at 55 % alpha.
    for i in 0..w.units.len() {
        let Some(pos) = w.grid.pos(i) else { continue };
        let u = &w.units[i];
        let r = cell_rect(pos);
        let ally = w.allies.contains(&i);
        let body = cx.art.first_body(&body_keys(&s.art[i], u));
        let palette = s.palettes.get(i).map_or("", |p| p.as_str());
        start_wanted(tokens, &mut cx.art, i, body.as_deref(), anim_now);
        // FX1-L1 rule I: the HP and life the board shows (the state before a step until its impact)
        let (hp, alive) = tokens.shown(i, u, w.units.len());
        let a = tokens.anims.get(i).unwrap_or(&still);
        let facing = if tokens.anims.get(i).is_some() { a.faces(anim_now, if instant { 0 } else { tuning.walk_ms }) } else { Facing::side(!ally) };
        if !alive && !still_dying(&mut cx.art, a, body.as_deref(), anim_now, instant, tuning.death_hold_ms) {
            ui.p.text_center(r.x as f32 + 16.0, r.y as f32 + 11.0, "x", BODY, th().dim);
            continue;
        }
        let (flash, shake) = flash_of(flashes, i, now, tuning, reduce || instant);
        let tp = geo.token_px;
        // A moving unit slides from its old tile.
        let (mut dx, mut dy) = (0, 0);
        if let Some((from, t0)) = a.slide {
            let (t, ms) = (anim_now.saturating_sub(t0), a.slide_ms(tuning.walk_ms));
            if t < ms && !instant {
                (dx, dy) = geo.slide(from, pos, 1.0 - t as f32 / ms.max(1) as f32);
            }
        }
        // FX1-L2: a miss's target steps aside and back
        let (sx, sy) = fx::sidestep_of(sidesteps, i, anim_now);
        let tr = geo.token_rect(pos, shake + dx + sx, dy + sy);
        let ring = if ally { ALLY } else { th().foe };
        if alive {
            // VISCLAR1-D: the ellipse in its intent's colour (`ring_intent`), else its side's
            let foot = ring_intent(rounds, i, ally, layers.glyphs, show_tells).map_or(ring, intent_ring_color);
            ui.p.ellipse(tr.x as f32 + tp as f32 / 2.0, (tr.y + tp) as f32 - 2.5, tp as f32 * 0.36, tp as f32 * 0.11, rgba(foot[0], foot[1], foot[2], tuning.foot_alpha));
        }
        let hover = ui.hover(r) && !popup;
        let outline = if Some(i) == sel || Some(i) == talker { Some(th().gold) } else if hover { Some(rgba(255, 255, 255, tuning.hover_outline_alpha)) } else { windup_outline(a, tuning) };
        let looks = status.iter().find(|x| x.unit == i).map_or(&[][..], |x| &x.looks[..]);
        let alpha = fx::token_alpha(looks, tuning);
        let frame = body.as_deref().and_then(|b| token_frame(&mut cx.art, b, palette, &a.player, anim_now, tp, facing).map(|(f, img)| {
            ui.p.image_alpha(img, tr, alpha);
            f
        }));
        match (frame, body.as_deref()) {
            (Some(f), Some(b)) => {
                if let Some((col, ta)) = fx::tint_of(looks, anim_now, reduce, tuning) {
                    status_tint(&mut ui.p, &mut cx.art, (b, palette, &f), tr, facing, col, scaled(ta, alpha));
                }
                if flash {
                    hit_flash(&mut ui.p, &mut cx.art, b, palette, &f, tr, facing);
                }
                if let Some(o) = outline.and_then(|col| token_outline(&mut cx.art, b, palette, &f, tp, col, facing)) {
                    ui.p.image_alpha(o, Rect::new(tr.x - 1, tr.y - 1, tp + 2, tp + 2), alpha);
                }
            }
            _ => {
                let ring = with_alpha(ring, scaled(ring[3], alpha));
                ui.p.panel(tr.inset(4), ring, outline.unwrap_or(ring), 6.0);
                ui.p.text_center(tr.x as f32 + tp as f32 / 2.0, r.y as f32 + 11.0 + dy as f32, &u.name.chars().next().unwrap_or('?').to_string(), BODY, th().bg);
                if flash {
                    ui.p.panel(tr.inset(4), fx::FLASH, fx::FLASH, 6.0);
                }
            }
        }
        if !alive {
            continue;
        }
        // VISCLAR1 VC2, VC3: under the round intent the bars of the focus set (Alt or `all_bars`: every unit's)
        if bars_shown(overlay.is_some(), &layers, alt, &focus, i) {
            hp_bar(&mut ui.p, Rect::new(r.x + 3 + dx, r.y + TILE - 3 + dy, TILE - 6, 2), hp, u.max_hp, if ally { th().good } else { th().foe });
            // RULESET1-U slice 5: the poise bar over the HP bar (rule poise)
            if let Some(st) = poise_shown(w, i, &layers) {
                poise_bar(&mut ui.p, Rect::new(r.x + 3 + dx, r.y + TILE - 6 + dy, TILE - 6, 2), &st, &w.ruleset.numbers);
            }
        }
        let tip = unit_tip(s, i, sel, tells, rounds, classes);
        ui.push(format!("unit.{i}"), "unit", format!("{} {}/{}", u.name, u.hp, u.max_hp), r, true, Some(i) == sel, Some(Action::UnitClick(i)), tip);
    }
    if show_tells {
        for r in tells {
            let Some(from) = w.grid.pos(r.unit) else { continue };
            // VISCLAR1 AR5, decision "Battle view answers": under the round intent no tell arrow (the target arrows
            // replace it) and no first-move chip (the round's intent marker is enough)
            if overlay.is_some() {
                continue;
            }
            let (fx, fy) = center(from);
            let to = r.line.target.and_then(|t| w.grid.pos(t)).or((r.line.cell != emberwake_battle::grid::NO_CELL).then_some(r.line.cell));
            if let Some(to) = to.filter(|c| *c != from) {
                let (tx, ty) = center(to);
                let (dx, dy) = (tx - fx, ty - fy);
                let len = (dx * dx + dy * dy).sqrt().max(1.0);
                // From the token's edge to just short of the target's.
                let (sx, sy, ex, ey) = (fx + dx / len * 10.0, fy + dy / len * 10.0, tx - dx / len * 9.0, ty - dy / len * 9.0);
                ui.p.arrow(sx, sy, ex, ey, rgba(240, 100, 92, 200), stage::arrow_dash(r.line.kind), 4.0);
            }
            let chip = Rect::new(GRID_X + from.x * TILE + 1, GRID_Y + from.y * TILE + 1, 10, 10);
            intent_chip(ui, cx, r.line.kind, chip);
        }
    }
    // RULESET1-U slice 4 (UI5): each unit's intent glyph at its cell's top right (an enemy's only while the tells show);
    // VISCLAR1-D: the ring tells the intent, the plate shows while Alt is held
    for r in rounds.iter().filter(|_| layers.glyphs && alt) {
        let (Some(i), Some(at)) = (r.round_intent, w.grid.pos(r.unit)) else { continue };
        if w.enemies.contains(&r.unit) && !show_tells {
            continue;
        }
        intent_glyph(&mut ui.p, &w.cat, i, Rect::new(GRID_X + at.x * TILE + TILE - 11, GRID_Y + at.y * TILE + 1, 10, 10));
    }
    // HOOKS1-UI slice 4: the push forecasts above the tokens (a collision mark sits between two of them)
    draw_pushes(&mut ui.p, pushes, &Geom::new(View::Flat, tuning));
    // RULESET1-U slice 5 (P7): a BREAK plate over each unit broken this round
    for i in broken_units(w, &layers) {
        let at = w.pos(i);
        break_plate(&mut ui.p, (GRID_X + at.x * TILE + TILE / 2) as f32, (GRID_Y + at.y * TILE + 2) as f32);
    }
    // FX1 4.1: the air layer over the units, before the floats (STATUSFX1 S6: the status loops first)
    draw_status_loops(&mut ui.p, &status, Layer::Air, &geo, tuning, anim_now, reduce);
    live.draw(&mut ui.p, anim_now, Layer::Air);
    for f in floats {
        if let Some(pos) = w.grid.pos(f.unit) {
            let t = now.duration_since(f.born).as_millis() as f32 / f.ms.max(1) as f32;
            let (cx, cy) = center(pos);
            float_text(ui, &f.text, f.color, cx, cy - 14.0 - tuning.float_rise_px as f32 * t);
        }
    }
    ui.p.shift(-shift.0, -shift.1);
    if shift != (0, 0) {
        ui.p.clip(None);
    }
    tile_hover
}

/// An intent chip (INTENT1): the kind's icon, else a lettered plate.
fn intent_chip(ui: &mut Ui<Action>, cx: &mut Cx, kind: &str, chip: Rect) {
    match cx.art.image(&emberwake_content::art::intent(kind), 10, 10) {
        Some(img) => ui.p.image(img, chip),
        None => {
            ui.p.panel(chip, th().game("foe_chip", rgba(48, 28, 38, 230)), th().foe, 2.0);
            ui.p.text_center(chip.x as f32 + 5.0, chip.y as f32 + 1.0, &kind[..1].to_uppercase(), SMALL, th().text);
        }
    }
}

/// A floating number with its shadow, centred on x at line top y; transient (TEXTFIT2: the text audit skips it).
fn float_text(ui: &mut Ui<Action>, text: &str, col: Rgba, x: f32, y: f32) {
    ui.p.transient(|p| {
        p.text_center(x + 0.6, y + 0.6, text, TITLE, rgba(0, 0, 0, 220));
        p.text_center(x, y, text, TITLE, col);
    });
}

/// One unit on the Stage this frame (STAGE1 3.6, 3.8).
struct StageUnit {
    i: usize,
    pos: V2,
    ally: bool,
    /// FX1-L1 rule I: the life and HP the board shows (`Tokens::shown`).
    alive: bool,
    hp: i32,
    /// Its sprite draws (standing, or still playing its death clip); else the dead marker shows.
    standing: bool,
    body: Option<String>,
    palette: String,
    /// DIR4-G: the facing drawn this frame (`UnitAnim::faces`).
    facing: Facing,
    flash: bool,
    /// The foot point with the slide.
    foot: (i32, i32),
    /// The drawn sprite square (slide and hit shake).
    tr: Rect,
    /// The body's solid rect at its place (slide, no shake): the feet pass's rect and half of its widget rect.
    body_rect: Rect,
    /// STATUSFX1: the status looks it draws (none once it has fallen).
    looks: Vec<fx::Drawn>,
}

/// A 1 px edged plate (the Stage's ground markers, 3.7 pass 3).
pub(super) fn plate(p: &mut Painter, r: Rect, fill: Rgba, edge: Rgba) {
    p.fill(r, fill);
    p.fill(Rect::new(r.x, r.y, r.w, 1), edge);
    p.fill(Rect::new(r.x, r.y + r.h - 1, r.w, 1), edge);
    p.fill(Rect::new(r.x, r.y + 1, 1, r.h - 2), edge);
    p.fill(Rect::new(r.x + r.w - 1, r.y + 1, 1, r.h - 2), edge);
}

/// A cell plate inset `ix` across and `iy` down.
pub(super) fn inset2(r: Rect, ix: i32, iy: i32) -> Rect {
    Rect::new(r.x + ix, r.y + iy, r.w - 2 * ix, r.h - 2 * iy)
}

pub(super) fn with_alpha(c: Rgba, a: u8) -> Rgba {
    [c[0], c[1], c[2], a]
}

/// STAGE1 option B, the Stage view: the board tilted to 32 x 24 cells, run as `stage::passes` (backdrop, ground,
/// markers, ground arrows, shadows, front face, the feet-sorted blocks, props and units, the outlines through what
/// covers a unit, the top: HP bars, chips, the exit sign, floats). Cell widgets first, then the units' in feet order
/// (front-most last, so it takes the click). `set`: the world's stage set (LOOK1-L1; "" = `common_stage`). Returns the
/// unit the mouse is over on the board.
pub(super) fn draw_stage(b: Board, set: &str, ui: &mut Ui<Action>, cx: &mut Cx, tuning: &Tuning, now: Instant) -> Option<usize> {
    let Board { s, tokens, floats, flashes, popup, anim_ms, sel, talker, planning, tells, show_tells, card_hover, inspect, focus, fx: live, shift, sidesteps, overlay, rounds, layers, alt, marks, classes, pushes, hold } = b;
    let w = &s.w;
    // VISCLAR1 AR3: the arrows of the selected and the inspected unit (Alt: all)
    let shown = overlay.map(|o| o.shown(&layers, sel, inspect, alt));
    let geo = Geom::new(View::Stage, tuning);
    let tp = geo.token_px;
    let anim_now = anim_ms as u64;
    let instant = cx.settings.v["skip_anim"].as_bool().unwrap_or(false);
    let reduce = cx.settings.reduce_motion();
    let still = UnitAnim::default();
    ui.p.panel(Rect::new(GRID_X - 2, GRID_Y - 2, GRID_W * TILE + 4, GRID_H * TILE + 4), th().game("board_tray", rgb(8, 10, 16)), th().edge, RADIUS);

    // The units' places this frame (STATUSFX1: and their status looks).
    let wall = tokens.wall_ms(now);
    for a in &mut tokens.anims {
        a.status_fx.clear();
    }
    let mut units: Vec<StageUnit> = Vec::new();
    for i in 0..w.units.len() {
        let Some(pos) = w.grid.pos(i) else { continue };
        let u = &w.units[i];
        let ally = w.allies.contains(&i);
        let body = cx.art.first_body(&body_keys(&s.art[i], u));
        let palette = s.palettes.get(i).cloned().unwrap_or_default();
        start_wanted(tokens, &mut cx.art, i, body.as_deref(), anim_now);
        let (hp, alive) = tokens.shown(i, u, w.units.len());
        let a = tokens.anims.get(i).unwrap_or(&still);
        let facing = if tokens.anims.get(i).is_some() { a.faces(anim_now, if instant { 0 } else { tuning.walk_ms }) } else { Facing::side(!ally) };
        let standing = alive || still_dying(&mut cx.art, a, body.as_deref(), anim_now, instant, tuning.death_hold_ms);
        let (mut dx, mut dy) = (0, 0);
        if let Some((from, t0)) = a.slide {
            let (t, ms) = (anim_now.saturating_sub(t0), a.slide_ms(tuning.walk_ms));
            if t < ms && !instant {
                (dx, dy) = geo.slide(from, pos, 1.0 - t as f32 / ms.max(1) as f32);
            }
        }
        let (flash, shake) = flash_of(flashes, i, now, tuning, reduce || instant);
        let (fx, fy) = geo.foot(pos);
        let rest = geo.token_rect(pos, dx, dy);
        // 3.8: the body's solid bounds of its idle frame 0 (animation frames do not move it); DIR4-HB: in the facing's
        // view (the front or back idle while it faces down or up, else the side idle with its left/right); no body:
        // the letter plate.
        let body_rect = match body.as_deref().and_then(|k| cx.art.body_bounds_facing(k, &palette, tp, facing)) {
            Some(bb) => Rect::new(rest.x + bb.x, rest.y + bb.y, bb.w, bb.h),
            None => rest.inset(4),
        };
        // FX1-L2: a miss's target steps aside and back (its sprite only; its foot keeps its place in the draw order)
        let (sx, sy) = fx::sidestep_of(sidesteps, i, anim_now);
        let looks = status_draws(tokens, i, alive, wall, anim_now, tuning);
        units.push(StageUnit { i, pos, ally, alive, hp, standing, body, palette, facing, flash, foot: (fx + dx, fy + dy), tr: geo.token_rect(pos, dx + shake + sx, dy + sy), body_rect, looks });
    }
    let status: Vec<StatusOn> = units.iter().filter(|t| t.alive).map(|t| StatusOn { unit: t.i, foot: (t.foot.0 as f32, t.foot.1 as f32), left: t.facing.left, looks: t.looks.clone() }).collect();
    // 3.7 pass 7: blocks, standing props and units sorted by their feet.
    let mut items = stage::board_items(&mut cx.art, &w.grid);
    items.extend(units.iter().filter(|t| t.standing).map(|t| stage::unit_item(t.i, t.foot, t.body_rect)));
    let order = stage::feet_order(&items);
    let unit_of = |i: usize| units.iter().find(|t| t.i == i);
    // 3.8: a standing unit's widget rect; the front-most under the mouse is the one a click reaches.
    let widget_rect = |t: &StageUnit| stage::unit_rect(geo.cell_rect(t.pos), t.body_rect);
    let board_hover = if ui.live && !popup {
        order.iter().rev().find_map(|p| match p.item {
            Item::Unit(i) => unit_of(i).filter(|t| t.alive && widget_rect(t).contains(ui.mouse.0, ui.mouse.1)).map(|t| t.i),
            _ => None,
        })
    } else {
        None
    };
    let hovered = board_hover.or(card_hover);
    // VC2: the unit under the mouse joins the focus set
    let mut focus = focus.to_vec();
    focus.extend(hovered);
    // The cell a click would reach (the hovered unit's, else the one under the mouse) shows the hover plate.
    let hover_cell = if planning && sel.is_some() && ui.live && !popup {
        board_hover.and_then(|u| w.grid.pos(u)).or_else(|| geo.cell_at(ui.mouse.0, ui.mouse.1))
    } else {
        None
    };
    let foot_f = |c: V2| {
        let (x, y) = geo.foot(c);
        (x as f32, y as f32)
    };
    let mut frames: Vec<(usize, ClipFrame)> = Vec::new();

    // FX1 4.1: passes 1-9 inside the board's clip, shifted by the camera (the clip stays put)
    ui.p.clip(Some(stage::INTERIOR));
    ui.p.shift(shift.0, shift.1);
    for pass in stage::passes() {
        match pass {
            // HOOKS1 ZN3: a data zone row keys its own backdrop and gradient ground (a code zone ignores any row)
            Pass::Backdrop => stage::draw_backdrop(&mut ui.p, &mut cx.art, &w.grid, &s.def.zone, &w.cat.zones),
            Pass::Ground => stage::draw_ground(&mut ui.p, &mut cx.art, &w.grid, set, tuning.grid_alpha),
            Pass::Markers => {
                // HOOKS1-UI slice 1: the ground hazards and the arena's struck cells, under the hover and the tells
                draw_hook_marks(&mut ui.p, marks, &geo);
                if let Some(h) = hold {
                    draw_hold_tiles(&mut ui.p, h, &geo);
                }
                if let Some(c) = hover_cell {
                    plate(&mut ui.p, geo.cell_rect(c), rgba(255, 255, 255, 30), rgba(255, 255, 255, 160));
                }
                // INTENT1 area tells: the tiles an enemy's area attack will hit.
                if show_tells {
                    for r in tells.iter().filter(|r| r.line.kind == "area") {
                        let cells = s.resolver.scheduled.iter().find_map(|a| match &a.command.kind {
                            CmdKind::Area { cells, aim, .. } if a.unit == r.unit && *aim == r.line.cell => Some(cells.clone()),
                            _ => None,
                        });
                        for c in cells.unwrap_or_default().into_iter().filter(|c| w.grid.is_valid(*c)) {
                            plate(&mut ui.p, geo.cell_rect(c).inset(1), rgba(240, 100, 92, 70), rgba(240, 100, 92, 230));
                        }
                    }
                }
                if let Some(o) = &shown {
                    // RULESET1-U slice 3: the round intent's overlays on the ground (the moves' destination plates)
                    draw_overlay(&mut ui.p, o, &geo);
                } else if let Some(h) = sel.filter(|h| w.units[*h].is_active() && planning) {
                    let st = s.plan.compile(w, h);
                    let mut from = w.pos(h);
                    for e in &st.entries {
                        if e.pos_after != from && e.pos_after.x >= 0 {
                            let (a, b) = (foot_f(from), foot_f(e.pos_after));
                            stage::wide_line(&mut ui.p, a.0, a.1 - 3.0, b.0, b.1 - 3.0, th().gold, 2.0, 0.0);
                            ui.p.panel(inset2(geo.cell_rect(e.pos_after), 3, 2), rgba(244, 198, 84, 40), th().gold, 2.0);
                            from = e.pos_after;
                        }
                    }
                }
                // ENCOUNTER2 H7: a hold escape's shut gate plates its exits red (layer `gate`)
                let (_, fill, edge) = exit_sign(hold);
                for c in w.grid.exits.iter().filter(|c| w.grid.is_valid(**c)) {
                    plate(&mut ui.p, inset2(geo.cell_rect(*c), 3, 2), with_alpha(fill, 90), with_alpha(edge, 200));
                }
            }
            // VISCLAR1 AR4, AR5: under the round intent the target arrows replace the tell arrows and the enemy attack
            // lines (classic keeps them)
            Pass::Arrows if shown.is_some() => {
                if let Some(o) = &shown {
                    draw_arrows(&mut ui.p, &o.arrows, &geo);
                }
            }
            Pass::Arrows => {
                let col = rgba(240, 100, 92, 210);
                let mut drawn: Vec<(V2, V2)> = Vec::new();
                if show_tells {
                    for r in tells {
                        let Some(from) = w.grid.pos(r.unit) else { continue };
                        let to = r.line.target.and_then(|t| w.grid.pos(t)).or((r.line.cell != emberwake_battle::grid::NO_CELL).then_some(r.line.cell));
                        if let Some(to) = to.filter(|c| *c != from) {
                            if let Some((x0, y0, x1, y1)) = stage::ground_arrow(foot_f(from), foot_f(to)) {
                                stage::wide_arrow(&mut ui.p, x0, y0, x1, y1, col, 1.5, stage::arrow_dash(r.line.kind), 4.0);
                            }
                            drawn.push((from, to));
                        }
                    }
                }
                if tuning.intent_arrows && planning {
                    let mut seen = Vec::new();
                    for a in &s.resolver.scheduled {
                        if !w.enemies.contains(&a.unit) || seen.contains(&a.unit) {
                            continue;
                        }
                        if let CmdKind::Attack { target: Some(t), .. } = a.command.kind {
                            seen.push(a.unit);
                            let (from, to) = (w.pos(a.unit), w.pos(t));
                            if from == to || drawn.contains(&(from, to)) {
                                continue;
                            }
                            if let Some((x0, y0, x1, y1)) = stage::ground_arrow(foot_f(from), foot_f(to)) {
                                stage::wide_arrow(&mut ui.p, x0, y0, x1, y1, col, 1.5, 0.0, 4.0);
                            }
                        }
                    }
                }
            }
            Pass::Shadows => {
                stage::draw_prop_shadows(&mut ui.p, &mut cx.art, &w.grid);
                for t in units.iter().filter(|t| t.alive) {
                    let (fx, fy) = (t.foot.0 as f32, t.foot.1 as f32);
                    ui.p.ellipse(fx + 1.0, fy, 0.2 * tp as f32, 2.4, rgba(8, 10, 6, tuning.shadow_alpha));
                    // VISCLAR1-D (mock `intent_ring`): the ring in its intent's colour, heavier than the side ring it
                    // replaces (`ring_intent`: the side's stays where no intent shows)
                    match ring_intent(rounds, t.i, t.ally, layers.glyphs, show_tells) {
                        Some(intent) => {
                            let col = intent_ring_color(intent);
                            ui.p.ellipse_ring(fx, fy, 0.25 * tp as f32, 3.2, col);
                            ui.p.ellipse_ring(fx, fy, 0.25 * tp as f32 - 0.5, 2.7, col);
                        }
                        None => ui.p.ellipse_ring(fx, fy, 0.25 * tp as f32, 3.2, with_alpha(if t.ally { ALLY } else { th().foe }, tuning.foot_ring_alpha)),
                    }
                }
            }
            Pass::Face => stage::draw_face(&mut ui.p, &mut cx.art, &w.grid, set),
            Pass::Feet => {
                for p in &order {
                    match p.item {
                        Item::Block(c) => stage::draw_block(&mut ui.p, &mut cx.art, &w.grid, set, c),
                        Item::Prop(c) => stage::draw_prop(&mut ui.p, &mut cx.art, &w.grid, c),
                        Item::Unit(i) => {
                            let Some(t) = unit_of(i) else { continue };
                            let a = tokens.anims.get(i).unwrap_or(&still);
                            let ring = if t.ally { ALLY } else { th().foe };
                            let outline = if Some(i) == sel || Some(i) == talker { Some(th().gold) } else if hovered == Some(i) { Some(rgba(255, 255, 255, tuning.hover_outline_alpha)) } else { windup_outline(a, tuning) };
                            // STATUSFX1: a stealthed token (sprite and outline) at 55 % alpha
                            let alpha = fx::token_alpha(&t.looks, tuning);
                            let frame = t.body.as_deref().and_then(|b| token_frame(&mut cx.art, b, &t.palette, &a.player, anim_now, tp, t.facing).map(|(f, img)| {
                                ui.p.image_alpha(img, t.tr, alpha);
                                f
                            }));
                            match (frame, t.body.as_deref()) {
                                (Some(f), Some(b)) => {
                                    // STATUSFX1: the tint right after the sprite, under the hit flash (FX1 4.1)
                                    if let Some((col, ta)) = fx::tint_of(&t.looks, anim_now, reduce, tuning) {
                                        status_tint(&mut ui.p, &mut cx.art, (b, &t.palette, &f), t.tr, t.facing, col, scaled(ta, alpha));
                                    }
                                    // FX1 4.1: the hit flash right after its unit's sprite
                                    if t.flash {
                                        hit_flash(&mut ui.p, &mut cx.art, b, &t.palette, &f, t.tr, t.facing);
                                    }
                                    if let Some(o) = outline.and_then(|col| token_outline(&mut cx.art, b, &t.palette, &f, tp, col, t.facing)) {
                                        ui.p.image_alpha(o, Rect::new(t.tr.x - 1, t.tr.y - 1, tp + 2, tp + 2), alpha);
                                    }
                                    frames.push((i, f));
                                }
                                _ => {
                                    let ring = with_alpha(ring, scaled(ring[3], alpha));
                                    ui.p.panel(t.tr.inset(4), ring, outline.unwrap_or(ring), 6.0);
                                    let letter = w.units[i].name.chars().next().unwrap_or('?').to_string();
                                    ui.p.text_center(t.tr.x as f32 + tp as f32 / 2.0, (t.tr.y + tp / 2) as f32 - 5.0, &letter, BODY, th().bg);
                                    if t.flash {
                                        ui.p.panel(t.tr.inset(4), fx::FLASH, fx::FLASH, 6.0);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Pass::Through => {
                // STAGE1-O: by default only the selected unit shows through; Alt held outlines every unit
                let reveal = stage::Reveal::of(&cx.settings.v["see_through"], cx.held.alt);
                for (k, p) in order.iter().enumerate() {
                    let Item::Unit(i) = p.item else { continue };
                    let Some(t) = unit_of(i).filter(|t| t.alive) else { continue };
                    let (Some(b), Some((_, f))) = (t.body.as_deref(), frames.iter().find(|(j, _)| *j == i)) else { continue };
                    let col = match stage::through(&order, k, Some(i) == sel, hovered == Some(i), reveal) {
                        Some(Through::Side) => with_alpha(if t.ally { ALLY } else { th().foe }, 230),
                        Some(Through::Selected) => th().gold,
                        Some(Through::Hovered) => rgba(255, 255, 255, tuning.hover_outline_alpha),
                        None => continue,
                    };
                    if let Some(o) = token_outline(&mut cx.art, b, &t.palette, f, tp, col, t.facing) {
                        ui.p.image_alpha(o, Rect::new(t.tr.x - 1, t.tr.y - 1, tp + 2, tp + 2), fx::token_alpha(&t.looks, tuning));
                    }
                }
            }
            Pass::Top => {
                // HOOKS1-UI slice 4: the push forecasts above the tokens
                draw_pushes(&mut ui.p, pushes, &geo);
                for t in &units {
                    let u = &w.units[t.i];
                    let (fx, fy) = t.foot;
                    if !t.standing {
                        ui.p.text_center(fx as f32, fy as f32 - ui.p.line_h(BODY) / 2.0, "x", BODY, th().dim);
                    } else if t.alive && bars_shown(overlay.is_some(), &layers, alt, &focus, t.i) {
                        // VISCLAR1 VC2, VC3: under the round intent the focus set's bars (Alt or `all_bars`: all)
                        hp_bar(&mut ui.p, Rect::new(fx - 12, fy + 3, 24, 2), t.hp, u.max_hp, if t.ally { th().good } else { th().foe });
                        // RULESET1-U slice 5: the poise bar under the HP bar (rule poise)
                        if let Some(st) = poise_shown(w, t.i, &layers) {
                            poise_bar(&mut ui.p, Rect::new(fx - 12, fy + 6, 24, 2), &st, &w.ruleset.numbers);
                        }
                    }
                }
                // Decision "Battle view answers": the first-move chip only in classic, which has no round intent marker
                if show_tells && overlay.is_none() {
                    for r in tells {
                        let Some(t) = units.iter().find(|t| t.i == r.unit) else { continue };
                        let chip = Rect::new(t.foot.0 - 15, (t.foot.1 as f32 - 0.95 * tp as f32 - 2.0).round() as i32, 10, 10);
                        intent_chip(ui, cx, r.line.kind, chip);
                    }
                }
                // RULESET1-U slice 4 (UI5): each unit's intent glyph right of where the tell chip sits (an enemy's only
                // while the tells show); VISCLAR1-D: the ring tells the intent, the plate shows while Alt is held
                for r in rounds.iter().filter(|_| layers.glyphs && alt) {
                    let (Some(i), Some(t)) = (r.round_intent, units.iter().find(|t| t.i == r.unit && t.alive)) else { continue };
                    if w.enemies.contains(&r.unit) && !show_tells {
                        continue;
                    }
                    intent_glyph(&mut ui.p, &w.cat, i, Rect::new(t.foot.0 + 5, (t.foot.1 as f32 - 0.95 * tp as f32 - 2.0).round() as i32, 10, 10));
                }
                // RULESET1-U slice 5 (P7): a BREAK plate over each unit broken this round, above its chips
                for i in broken_units(w, &layers) {
                    let Some(t) = units.iter().find(|t| t.i == i && t.alive) else { continue };
                    break_plate(&mut ui.p, t.foot.0 as f32, t.foot.1 as f32 - 0.95 * tp as f32 - 13.0);
                }
                for c in w.grid.exits.iter().filter(|c| w.grid.is_valid(**c)) {
                    let r = geo.cell_rect(*c);
                    let (cx_, cy_) = (r.x + r.w / 2, r.y + r.h / 2);
                    let sign = Rect::new(cx_ - 11, cy_ - 2 - 5, 22, 9);
                    // ENCOUNTER2 H7: a hold escape's exit reads SHUT or OPEN (layer `gate`)
                    let (word, fill, edge) = exit_sign(hold);
                    ui.p.panel(sign, fill, edge, 2.0);
                    ui.p.text_center(cx_ as f32, sign.y as f32 + (sign.h as f32 - ui.p.line_h(SMALL)) / 2.0, word, SMALL, th().bg);
                }
                for f in floats {
                    let Some(pos) = w.grid.pos(f.unit) else { continue };
                    let t = now.duration_since(f.born).as_millis() as f32 / f.ms.max(1) as f32;
                    let (x, y) = geo.chest(pos);
                    float_text(ui, &f.text, f.color, x as f32, y as f32 - 14.0 - tuning.float_rise_px as f32 * t);
                }
            }
        }
        // FX1 4.1: the ground layer after pass 4 (the ground arrows), the air layer after pass 8 (before the floats);
        // STATUSFX1 S6: on each the status loops first, then FX1's live effects
        match pass {
            Pass::Arrows => {
                draw_status_loops(&mut ui.p, &status, Layer::Ground, &geo, tuning, anim_now, reduce);
                live.draw(&mut ui.p, anim_now, Layer::Ground);
            }
            Pass::Through => {
                draw_status_loops(&mut ui.p, &status, Layer::Air, &geo, tuning, anim_now, reduce);
                live.draw(&mut ui.p, anim_now, Layer::Air);
            }
            _ => {}
        }
    }
    ui.p.shift(-shift.0, -shift.1);
    ui.p.clip(None);

    // 3.8: the cell widgets (32 x 24), then the units' in feet order (front-most last).
    for gy in 0..GRID_H {
        for gx in 0..GRID_W {
            let c = V2::new(gx, gy);
            let mut tip: Tip = if w.grid.is_exit(c) { vec![("Exit".into(), th().good), ("A unit that steps here leaves the battle.".into(), th().text)] } else { Vec::new() };
            tip.extend(marks.tile_tip(c));
            tip.extend(hold.map(|h| h.tile_tip(w, c)).unwrap_or_default());
            ui.push(format!("tile.{gx}.{gy}"), "tile", w.grid.terrain_id(c), geo.cell_rect(c), planning, false, if w.occupant(c).is_none() { Some(Action::TileClick(c)) } else { None }, tip);
        }
    }
    for p in &order {
        let Item::Unit(i) = p.item else { continue };
        let Some(t) = unit_of(i).filter(|t| t.alive) else { continue };
        let u = &w.units[i];
        let tip = unit_tip(s, i, sel, tells, rounds, classes);
        ui.push(format!("unit.{i}"), "unit", format!("{} {}/{}", u.name, u.hp, u.max_hp), widget_rect(t), true, Some(i) == sel, Some(Action::UnitClick(i)), tip);
    }
    board_hover
}
