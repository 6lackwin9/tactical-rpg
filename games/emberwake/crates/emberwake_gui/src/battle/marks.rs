//! Board marks and their data: target arrows and focus (VISCLAR1), impact tracers (INPUT1-PB), ground hazards and the arena telegraph, the hold escape, push forecasts, unit meters, poise, break plates and hit forecasts.

use aegis_ui::theme::th;
use super::board::{inset2, plate, with_alpha};
use super::overlays::{ground_pt, strike_tile, Layers};
use super::{Action, GRID_X, GRID_Y, POISE, TILE, TRACER_ALPHA, TRACER_INSET, TRACER_MS};
use aegis_ui::canvas::{rgb, rgba, Rect, Rgba};
use crate::fx;
use crate::stage::{self, Geom, View};
use emberwake_battle::intent::Intent;
use aegis_ui::painter::Painter;
use aegis_ui::ui::{Tip, Ui, BODY, RADIUS, SMALL, TITLE};
use emberwake_battle::command::CmdKind;
use emberwake_battle::grid::{V2, GRID_H, GRID_W};
use emberwake_battle::round_intent::{self as ri, RoundTarget};
use emberwake_battle::session::Session;
use emberwake_battle::skill::Kind;
use emberwake_battle::world::World;
use serde_json::{json, Value};

// ---- VISCLAR1 slice A: the target arrows, the bars in focus, Party foresight off (INPUT1 10) ------------------------

/// VISCLAR1 AR1: one unit's target arrow, a polyline of tiles: its tile, the end of its moves before its first command
/// when that move shows (AR2), the target (a unit's tile or the target tile; none for a self target, which ends at the
/// move's end). Two points or more.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Arrow {
    pub(super) unit: usize,
    pub(super) points: Vec<V2>,
    pub(super) ally: bool,
}

/// AR4: the arrows' alpha (of 255), dash (px on, px off), head (px) and width (Flat, Stage).
const ARROW_ALPHA: u8 = 90;
const ARROW_DASH: (f32, f32) = (4.0, 3.0);
const ARROW_HEAD: f32 = 4.0;
const ARROW_W_FLAT: f32 = 1.0;
const ARROW_W_STAGE: f32 = 1.5;

/// VC4: the debug settings key of "Party foresight off" (in no `default_settings`, VC5).
pub(crate) const FORESIGHT_OFF: &str = "debug_party_foresight_off";

/// VC5: whether the party's foresight reveals draw as unrevealed: the debug key, read only when `debug` (called with
/// `cfg!(debug_assertions)`), so a hand-edited settings file cannot turn it on in a release build.
pub(crate) fn party_foresight_off(settings: &Value, debug: bool) -> bool {
    debug && settings.get(FORESIGHT_OFF).and_then(Value::as_bool).unwrap_or(false)
}

/// VC4: the records `v` as the screen shows them; with `off` (Party foresight off) a plan that only a party foresight
/// passive revealed reads unrevealed (no plan lines, no readers); a reveal by enrage or by the battle's `plan_reveal`
/// stays. Display only: the kit's records are unchanged.
pub(crate) fn seen_intents(w: &World, mut v: Vec<Intent>, off: bool) -> Vec<Intent> {
    if !off {
        return v;
    }
    let reveal_all = emberwake_battle::intent::badges_on(w) && w.plan_reveal;
    for r in v.iter_mut().filter(|r| !r.readers.is_empty()) {
        r.readers.clear();
        if !r.enraged && !reveal_all {
            r.revealed = false;
            r.plan.clear();
        }
    }
    v
}

/// AR1, AR2: every unit's target arrow while the party plans (empty otherwise and in classic): a party member's from
/// its round target and its compiled plan (the move leg with the group `moves` on); an enemy's while the tells show,
/// to its round target (without a round intent: its tell's target), with the move leg only when its plan is revealed
/// (`tells` as the screen shows them, VC4) and the group `walks` is on. An after-move is no part of it (Q14: its
/// destination plate shows it).
pub(super) fn arrows_of(s: &Session, tells: &[Intent], rounds: &[Intent], layers: &Layers, show_tells: bool) -> Vec<Arrow> {
    let w = &s.w;
    if !ri::on(w) || !s.is_planning() {
        return Vec::new();
    }
    let on_field = |u: usize| w.units[u].is_active() && w.has(u);
    let tile_of = |t: RoundTarget, me: usize| match t {
        RoundTarget::Unit(u) if u != me && on_field(u) => Some(Some(w.pos(u))),
        RoundTarget::Unit(u) if u == me => Some(None),
        RoundTarget::Myself => Some(None),
        RoundTarget::Cell(c) if w.grid.is_valid(c) => Some(Some(c)),
        _ => None,
    };
    let mut out = Vec::new();
    let mut push = |unit: usize, ally: bool, leg: Option<V2>, to: Option<Option<V2>>| {
        let from = w.pos(unit);
        let mut points = vec![from];
        if let Some(end) = leg.filter(|c| *c != from) {
            points.push(end);
        }
        match to {
            Some(Some(t)) if points.last() != Some(&t) => points.push(t),
            // a self target ends at the move's end (nothing without a move); no target draws nothing
            Some(_) => {}
            None => return,
        }
        if points.len() >= 2 {
            out.push(Arrow { unit, points, ally });
        }
    };
    for &h in w.allies.iter().filter(|&&h| on_field(h)) {
        let target = w.round_states.get(&h).and_then(|r| r.target).or_else(|| rounds.iter().find(|r| r.unit == h).and_then(|r| r.round_target));
        let leg = layers.moves.then(|| party_leg(s, h));
        push(h, true, leg, target.and_then(|t| tile_of(t, h)));
    }
    if show_tells {
        for t in tells.iter().filter(|t| on_field(t.unit)) {
            let rec = rounds.iter().find(|r| r.unit == t.unit && r.round_intent.is_some());
            let to = match rec {
                Some(r) => r.round_target.and_then(|x| tile_of(x, t.unit)),
                None => t.line.target.filter(|u| on_field(*u)).map(|u| Some(w.pos(u))).or_else(|| (w.grid.is_valid(t.line.cell)).then_some(Some(t.line.cell))),
            };
            let leg = (layers.walks && t.revealed).then(|| enemy_leg(s, t.unit));
            push(t.unit, false, leg, to);
        }
    }
    out
}

/// AR1: where party member `h`'s moves before its first command end (its walks, then the carried move its first
/// command rides; Rest is no command); its own tile without one.
pub(super) fn party_leg(s: &Session, h: usize) -> V2 {
    let w = &s.w;
    let mut at = w.pos(h);
    for e in &s.plan.compile(w, h).entries {
        match emberwake_battle::skill::kind(&w.cat.skills[e.skill]) {
            Kind::Move => {
                if w.grid.is_valid(e.pos_after) {
                    at = e.pos_after;
                }
            }
            Kind::Rest => {}
            _ => {
                if w.grid.is_valid(strike_tile(e)) {
                    at = strike_tile(e);
                }
                break;
            }
        }
    }
    at
}

/// AR2: where a revealed enemy's moves before its first command end (its walks as the kit would walk them now,
/// `command::walk_preview`, then a carried Engage dash of its first command); its own tile without one.
fn enemy_leg(s: &Session, e: usize) -> V2 {
    let w = &s.w;
    let mut acts: Vec<&emberwake_battle::resolver::ScheduledAction> = s.resolver.scheduled.iter().filter(|a| a.unit == e && !a.cancelled).collect();
    acts.sort_by_key(|a| a.start);
    let mut at = w.pos(e);
    for a in acts {
        match (&a.command.kind, a.command.carry) {
            (CmdKind::Walk { path, expected_start, .. }, _) => {
                if let Some(end) = emberwake_battle::command::walk_preview(w, e, path, *expected_start).last() {
                    at = *end;
                }
            }
            (_, emberwake_battle::command::Carry::Engage(c)) => {
                at = c;
                break;
            }
            _ => break,
        }
    }
    at
}

/// VC2: the focus set whose token bars draw by default: the selected member, its round target when that is a unit,
/// the hovered unit (token or card) and the inspected unit.
pub(super) fn focus_set(w: &World, sel: Option<usize>, hovered: Option<usize>, inspect: Option<usize>) -> Vec<usize> {
    let target = sel.and_then(|h| w.round_states.get(&h)).and_then(|r| match r.target {
        Some(RoundTarget::Unit(t)) => Some(t),
        _ => None,
    });
    let mut out: Vec<usize> = Vec::new();
    for u in [sel, target, hovered, inspect].into_iter().flatten() {
        if !out.contains(&u) {
            out.push(u);
        }
    }
    out
}

/// VC2, VC3: whether unit `i`'s token bars draw: in classic always; under the round intent for the focus set, every
/// unit while Alt is held or the group `all_bars` is on.
pub(super) fn bars_shown(timeline: bool, layers: &Layers, alt: bool, focus: &[usize], i: usize) -> bool {
    !timeline || layers.all_bars || alt || focus.contains(&i)
}

/// AR4: a polyline of ground points dashed `ARROW_DASH` (the pattern runs on over the corners), its head of
/// `ARROW_HEAD` at the end.
fn draw_arrow_line(p: &mut Painter, pts: &[(f32, f32)], col: Rgba, width: f32) {
    dash_polyline(p, pts, col, width);
    if let [.., a, b] = pts {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let len = (dx * dx + dy * dy).sqrt();
        if len >= 0.5 {
            let (ux, uy) = (dx / len, dy / len);
            for side in [-1.0f32, 1.0] {
                let (c, s) = (0.866f32, 0.5 * side);
                let (bx, by) = (-(ux * c - uy * s), -(uy * c + ux * s));
                stage::wide_line(p, b.0, b.1, b.0 + bx * ARROW_HEAD, b.1 + by * ARROW_HEAD, col, width, 0.0);
            }
        }
    }
}

/// AR4, PB1: a polyline in `ARROW_DASH`'s dash (4 px on, 3 off), the dash running on over each corner.
fn dash_polyline(p: &mut Painter, pts: &[(f32, f32)], col: Rgba, width: f32) {
    let (on, off) = ARROW_DASH;
    let mut phase = 0.0f32;
    for seg in pts.windows(2) {
        let ((x0, y0), (x1, y1)) = (seg[0], seg[1]);
        let len = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();
        if len < 0.5 {
            continue;
        }
        let (ux, uy) = ((x1 - x0) / len, (y1 - y0) / len);
        let mut d = 0.0f32;
        while d < len {
            let at = (phase + d) % (on + off);
            let step = if at < on { on - at } else { on + off - at };
            let e = (d + step).min(len);
            if at < on {
                stage::wide_line(p, x0 + ux * d, y0 + uy * d, x0 + ux * e, y0 + uy * e, col, width, 0.0);
            }
            d = e;
        }
        phase = (phase + len) % (on + off);
    }
}

/// AR4: the shown arrows on the ground, under the tokens (Flat: after the ground overlays; Stage: the Arrows pass):
/// party GOLD, enemy FOE at `ARROW_ALPHA`; each leg leaves its tile's token and the last stops short of its target's
/// (`stage::ground_arrow`).
pub(super) fn draw_arrows(p: &mut Painter, arrows: &[Arrow], geo: &Geom) {
    let width = if geo.view == View::Stage { ARROW_W_STAGE } else { ARROW_W_FLAT };
    for a in arrows {
        let col = with_alpha(if a.ally { th().gold } else { th().foe }, ARROW_ALPHA);
        let at: Vec<(f32, f32)> = a.points.iter().map(|c| ground_pt(geo, *c)).map(|(x, y)| (x, y - 3.0)).collect();
        // the first leg leaves the unit's foot ring, the last stops short of the target's (a leg too short keeps its end)
        let mut pts = at.clone();
        let n = at.len();
        if n >= 2 {
            if let Some((x0, y0, _, _)) = stage::ground_arrow(at[0], at[1]) {
                pts[0] = (x0, y0);
            }
            if let Some((_, _, x1, y1)) = stage::ground_arrow(at[n - 2], at[n - 1]) {
                pts[n - 1] = (x1, y1);
            }
        }
        draw_arrow_line(p, &pts, col, width);
    }
}

// ---- INPUT1-PB: the impact tracers (INPUT1 PB1) ---------------------------------------------------------------------

/// INPUT1 PB1: the tracers of popped action `idx` (tiles by `at`): from its actor to each unit it landed on (an attack's
/// or an area's hit, damage or stagger; a support or an item on another unit, in the heal colour `heal` unless the item
/// harms), dashed to its target when it missed (MR8 c's protected flank, GD5's lost target), to the aim tile an area
/// left its hazard on, and from each riposting unit back to the actor; party GOLD, enemy FOE. None for a self-only
/// command, a cancelled one, or one that failed otherwise.
pub(super) fn traces_of(s: &Session, idx: usize, at: &dyn Fn(usize) -> Option<V2>, heal: Rgba) -> Vec<fx::Trace> {
    let w = &s.w;
    let mut out: Vec<fx::Trace> = Vec::new();
    let Some(a) = s.resolver.scheduled.get(idx).filter(|a| !a.cancelled) else { return out };
    let Some(from) = at(a.unit) else { return out };
    let c = &a.command;
    let side = |u: usize| if w.allies.contains(&u) { th().gold } else { th().foe };
    let put = |out: &mut Vec<fx::Trace>, from: V2, to: Option<V2>, col: Rgba, dashed: bool| {
        if let Some(to) = to.filter(|to| *to != from && !out.iter().any(|t| (t.from, t.to) == (from, *to))) {
            out.push(fx::Trace { from, to, col, dashed });
        }
    };
    let landed: Vec<usize> = c.hits.iter().map(|h| h.0).chain(c.staggers.iter().map(|st| st.target)).filter(|&u| u != a.unit).collect();
    match &c.kind {
        CmdKind::Attack { target, .. } => {
            let missed = c.evaded.or(if c.missed { *target } else { None });
            if let Some(t) = missed {
                put(&mut out, from, at(t), side(a.unit), true);
            }
            for &u in &landed {
                put(&mut out, from, at(u), side(a.unit), false);
            }
        }
        CmdKind::Area { aim, .. } => {
            for &u in &landed {
                put(&mut out, from, at(u), side(a.unit), false);
            }
            // HOOKS1 GH2: the hazard an area leaves lands on its aim tile (rule ground_hazards)
            if c.hazard.as_ref().is_some_and(|h| !h.at_self) && emberwake_battle::ground::on(w) && !c.failed && w.grid.is_valid(*aim) {
                put(&mut out, from, Some(*aim), side(a.unit), false);
            }
        }
        CmdKind::Support { target, .. } if *target != a.unit && !c.failed => put(&mut out, from, at(*target), heal, false),
        CmdKind::Item { target, item, used: true, .. } if *target != a.unit => put(&mut out, from, at(*target), if item.damage > 0 { side(a.unit) } else { heal }, false),
        _ => {}
    }
    for r in &c.reactions {
        if let Some(back) = at(r.defender) {
            put(&mut out, back, at(r.attacker), side(r.defender), false);
        }
    }
    out
}

/// PB1: a tracer's alpha `age` FX-clock ms after its impact: `TRACER_ALPHA` fading to 0 over `TRACER_MS`; under reduce
/// motion held at `TRACER_ALPHA`, then gone.
pub(super) fn tracer_alpha(age: u64, still: bool) -> u8 {
    if age >= TRACER_MS {
        0
    } else if still {
        TRACER_ALPHA
    } else {
        (TRACER_ALPHA as f32 * (1.0 - age as f32 / TRACER_MS as f32)).round() as u8
    }
}

/// PB1: the live tracers above the tokens, each a straight line between the two tokens' centres shortened by
/// `TRACER_INSET` at each end, 1 px on the Flat board and 1.5 px on the Stage, dashed (4 on, 3 off) for a miss.
pub(super) fn draw_tracers(p: &mut Painter, tracers: &[(fx::Trace, u64)], geo: &Geom, now: u64, still: bool) {
    let width = if geo.view == View::Stage { ARROW_W_STAGE } else { ARROW_W_FLAT };
    for (t, at) in tracers {
        let a = tracer_alpha(now.saturating_sub(*at), still);
        let ((x0, y0), (x1, y1)) = (fx::anchor(geo, t.from), fx::anchor(geo, t.to));
        let len = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();
        if a == 0 || len <= 2.0 * TRACER_INSET {
            continue;
        }
        let (ux, uy) = ((x1 - x0) / len, (y1 - y0) / len);
        let pts = [(x0 + ux * TRACER_INSET, y0 + uy * TRACER_INSET), (x1 - ux * TRACER_INSET, y1 - uy * TRACER_INSET)];
        let col = with_alpha(t.col, a);
        if t.dashed {
            dash_polyline(p, &pts, col, width);
        } else {
            stage::wide_line(p, pts[0].0, pts[0].1, pts[1].0, pts[1].1, col, width, 0.0);
        }
    }
}

/// The arrows for the agent link's state: [{unit, points [[x, y]], alpha}].
pub(super) fn arrows_json(arrows: &[Arrow]) -> Value {
    Value::Array(arrows.iter().map(|a| json!({"unit": a.unit, "points": a.points.iter().map(|c| json!([c.x, c.y])).collect::<Vec<_>>(), "alpha": ARROW_ALPHA})).collect())
}

// ---- HOOKS1-UI slice 1: ground hazards and the arena telegraph (HOOKS1 GH6, AH3) ------------------------------------

/// The arena telegraph's colour (struck cells and the strip badge).
const ARENA: Rgba = rgb(255, 146, 48);

/// One ground hazard as the party sees it (GH6: a foe's hidden hazard is left out).
pub(super) struct GroundMark {
    pub(super) cell: V2,
    id: String,
    name: String,
    tint: Rgba,
    glyph: String,
    /// One of the party's own hidden hazards (the foes do not see it): drawn fainter.
    pub(super) hidden: bool,
    last_round: i32,
    tip: Tip,
}

/// HOOKS1-UI slice 1: what the hook layers mark on the board and the strip: the ground hazards the party's side sees
/// (rule ground_hazards, layer `hazards`) and the arena hazards due this round (rule arena_hazards, layer `arena`; the
/// kit's tells, AH3). Classic has neither rule, so both stay empty whatever the flags say.
#[derive(Default)]
pub(super) struct HookMarks {
    pub(super) ground: Vec<GroundMark>,
    pub(super) arena: Vec<emberwake_battle::intent::ArenaTell>,
    /// Each arena hazard's effect in words, aligned with `arena`.
    arena_effects: Vec<String>,
    /// Decision "Battle view answers" (user 2026-11-03): the board draws the marks with their group on or while Alt is
    /// held; the hover lines read `ground` and `arena` whatever these say.
    show_ground: bool,
    pub(super) show_arena: bool,
}

impl HookMarks {
    pub(super) fn of(s: &Session, layers: &Layers, alt: bool) -> HookMarks {
        let w = &s.w;
        let mut out = HookMarks { show_ground: layers.hazards || alt, show_arena: layers.arena || alt, ..HookMarks::default() };
        if emberwake_battle::ground::on(w) {
            // the party's side sees what any of its members sees (GH6 is per side)
            if let Some(&viewer) = w.allies.first() {
                out.ground = w.grid.ground.iter().filter(|(_, p)| p.seen_by(w, viewer)).map(|(c, p)| ground_mark(w, *c, p)).collect();
            }
        }
        out.arena = emberwake_battle::intent::arena_tells(w);
        out.arena_effects = out.arena.iter().map(|t| w.arena.hazards.iter().find(|h| h.id == t.id).map(arena_effect_text).unwrap_or_default()).collect();
        out
    }

    fn ground_at(&self, c: V2) -> Option<&GroundMark> {
        self.ground.iter().find(|g| g.cell == c)
    }

    /// The tile's hover lines: its ground hazard, then each arena hazard that strikes it this round.
    pub(super) fn tile_tip(&self, c: V2) -> Tip {
        let mut tip: Tip = self.ground_at(c).map(|g| g.tip.clone()).unwrap_or_default();
        for (t, e) in self.arena.iter().zip(&self.arena_effects).filter(|(t, _)| t.cells.contains(&c)) {
            tip.push((format!("{}: strikes here this round", t.name), ARENA));
            tip.push((format!("At the end of {}", blocks_text(&t.blocks)), th().text));
            if !e.is_empty() {
                tip.push((e.clone(), th().text));
            }
        }
        tip
    }

    /// The strip block's arena lines: each hazard that strikes at the end of block `b` this round.
    pub(super) fn block_tip(&self, b: i32) -> Tip {
        let mut tip: Tip = Vec::new();
        for (t, e) in self.arena.iter().zip(&self.arena_effects).filter(|(t, _)| t.blocks.contains(&b)) {
            tip.push((format!("{} strikes at the end of block {b} ({} cell{})", t.name, t.cells.len(), if t.cells.len() == 1 { "" } else { "s" }), ARENA));
            if !e.is_empty() {
                tip.push((e.clone(), th().text));
            }
        }
        tip
    }

    /// The agent link's state: the marks as drawn (tiles as [x, y]).
    pub(super) fn json(&self) -> Value {
        json!({
            "ground": self.ground.iter().filter(|_| self.show_ground).map(|g| json!({"at": [g.cell.x, g.cell.y], "id": g.id, "name": g.name, "glyph": g.glyph, "hidden": g.hidden, "last_round": g.last_round})).collect::<Vec<_>>(),
            "arena": if self.show_arena { emberwake_battle::intent::arena_json(&self.arena) } else { json!([]) },
        })
    }
}

/// "block 3" or "blocks 3 and 6" or "blocks 2, 4 and 6".
fn blocks_text(blocks: &[i32]) -> String {
    let b: Vec<String> = blocks.iter().map(|b| b.to_string()).collect();
    match b.len() {
        0 => "no block".into(),
        1 => format!("block {}", b[0]),
        n => format!("blocks {} and {}", b[..n - 1].join(", "), b[n - 1]),
    }
}

/// An arena hazard's effect in words (AH2: damage through guard, not armour; the status roll; stagger; a push).
fn arena_effect_text(h: &emberwake_battle::arena::Hazard) -> String {
    let e = &h.effect;
    let mut bits: Vec<String> = Vec::new();
    if e.damage > 0 {
        bits.push(format!("{} damage", e.damage));
    }
    if e.damage_pct > 0 {
        bits.push(format!("{}% of max HP", e.damage_pct));
    }
    if !e.status.is_empty() {
        bits.push(format!("{} ({}% chance)", emberwake_rules::status::display_name(&e.status), e.potency));
    }
    if e.stagger > 0 {
        bits.push(format!("stagger {}", e.stagger));
    }
    if let Some(p) = e.push {
        let dir = match (p.dir.x, p.dir.y) {
            (1, _) => "east",
            (-1, _) => "west",
            (_, -1) => "north",
            _ => "south",
        };
        bits.push(format!("pushes {dir} {} tile{}", p.tiles, if p.tiles == 1 { "" } else { "s" }));
    }
    bits.join(", ")
}

/// A ground hazard's mark and hover lines (GH1: its row's numbers; GH7: the round it lasts through).
fn ground_mark(w: &World, cell: V2, p: &emberwake_battle::ground::Placed) -> GroundMark {
    let d = &p.def;
    let placer = w.units.get(p.by).map(|u| u.name.clone()).unwrap_or_default();
    let tint = aegis_ui::vignette::hex(&d.tint).unwrap_or(rgb(200, 120, 220));
    let mut tip: Tip = vec![(d.name.clone(), tint)];
    if !d.description.is_empty() {
        tip.push((d.description.clone(), th().text));
    }
    let mut does: Vec<String> = Vec::new();
    if d.damage > 0 {
        does.push(format!("{} damage", d.damage));
    }
    if !d.status.is_empty() {
        does.push(format!("{} ({}% chance)", emberwake_rules::status::display_name(&d.status), d.potency));
    }
    if !does.is_empty() {
        let when = match (d.trap, d.on.as_str()) {
            (true, _) => "When sprung",
            (_, "round_end") => "At round end",
            (_, "both") => "On entering and at round end",
            _ => "On entering",
        };
        tip.push((format!("{when}: {}", does.join(", ")), th().text));
    }
    if d.trap {
        tip.push((format!("Trap: stops the move of the first unit it affects{}, then is gone", if d.stop_moves { " and its later moves this round" } else { "" }), th().foe));
    } else if d.stop_moves {
        tip.push(("Stops the later moves this round of a unit that enters".into(), th().text));
    }
    if d.cover_guard > 0.0 {
        tip.push((format!("Cover {:.0}% against ranged hits", d.cover_guard * 100.0), th().text));
    }
    if d.move_cost > 0 {
        tip.push((format!("Move cost +{}", d.move_cost), th().text));
    }
    let affects = match d.affects.as_str() {
        "foes" => format!("Affects {placer}'s foes"),
        "friends" => format!("Affects {placer}'s side"),
        _ => "Affects every unit".into(),
    };
    tip.push((affects, th().dim));
    tip.push((format!("Left by {placer} in round {}; lasts through round {}", p.round, p.last_round), th().dim));
    if d.hidden {
        tip.push(("Hidden from the foes".into(), th().dim));
    }
    GroundMark { cell, id: d.id.clone(), name: d.name.clone(), tint, glyph: d.glyph.clone(), hidden: d.hidden, last_round: p.last_round, tip }
}

/// HOOKS1-UI slice 1: the ground hazards (a tinted plate with the row's glyph; the party's own hidden ones fainter) and
/// the arena's struck cells (an amber plate with a hatch) on the ground, under the tokens and the round overlays.
pub(super) fn draw_hook_marks(p: &mut Painter, m: &HookMarks, geo: &Geom) {
    let plate_of = |c: V2| match geo.view {
        View::Stage => inset2(geo.cell_rect(c), 1, 1),
        View::Flat => geo.cell_rect(c).inset(1),
    };
    for g in m.ground.iter().filter(|_| m.show_ground) {
        let r = plate_of(g.cell);
        let (fill, edge) = if g.hidden { (36, 120) } else { (70, 200) };
        plate(p, r, with_alpha(g.tint, fill), with_alpha(g.tint, edge));
        if !g.glyph.is_empty() {
            let cy = r.y as f32 + (r.h as f32 - p.line_h(SMALL)) / 2.0;
            p.text_center(r.x as f32 + r.w as f32 / 2.0, cy, &g.glyph, SMALL, with_alpha(th().text, if g.hidden { 150 } else { 235 }));
        }
    }
    let mut struck: Vec<V2> = Vec::new();
    for c in m.arena.iter().filter(|_| m.show_arena).flat_map(|t| &t.cells) {
        if !struck.contains(c) {
            struck.push(*c);
        }
    }
    for c in struck {
        let r = plate_of(c).inset(1);
        plate(p, r, with_alpha(ARENA, 34), with_alpha(ARENA, 190));
        // a hatch: two short strokes in the lower right corner, so a tinted ground hazard under it still reads
        let (x1, y1) = ((r.x + r.w) as f32 - 2.0, (r.y + r.h) as f32 - 2.0);
        for k in [5.0, 9.0] {
            p.line(x1 - k, y1, x1, y1 - k * 0.6, with_alpha(ARENA, 200), 1.0);
        }
    }
}

/// HOOKS1-UI slice 1: the arena badge on the strip block `r` (a hazard strikes at its end).
pub(super) fn arena_badge(p: &mut Painter, r: Rect) {
    let b = Rect::new(r.x + r.w - 7, r.y + 1, 6, 6);
    p.panel(b, ARENA, rgba(60, 26, 6, 255), 1.0);
}

// ---- ENCOUNTER2-U: the hold escape's pick, tiles and gate (ENCOUNTER2 H1, H3, H7) -----------------------------------

/// A hold escape's marks as drawn: the tiles (layer `hold`), the gate's state (layer `gate`), the holder and the label.
pub(super) struct HoldMarks {
    tiles: Vec<V2>,
    label: String,
    holder: Option<usize>,
    open: bool,
    show_tiles: bool,
    show_gate: bool,
}

impl HoldMarks {
    /// The session's hold gate (`World::hold`, the kit's `HoldGate`); None without a hold escape (classic and every other
    /// fight draw as before).
    pub(super) fn of(s: &Session, layers: &Layers) -> Option<HoldMarks> {
        let gate = s.w.hold.as_ref()?;
        Some(HoldMarks { tiles: gate.tiles.clone(), label: gate.label.clone(), holder: gate.holder, open: gate.is_open(&s.w), show_tiles: layers.hold, show_gate: layers.gate })
    }

    /// A tile's hover lines: a hold tile names its holder; an exit tells the gate's state (`HoldGate::note`'s words).
    pub(super) fn tile_tip(&self, w: &World, c: V2) -> Tip {
        let mut tip: Tip = Vec::new();
        if self.show_tiles && self.tiles.contains(&c) {
            let mut head = self.label.chars();
            tip.push((head.next().map(|f| f.to_uppercase().collect::<String>() + head.as_str()).unwrap_or_default(), th().gold));
            tip.push((
                match self.holder {
                    Some(h) => format!("{} holds it: the gate is open while they stand here, and they stay behind when the others escape.", w.units[h].name),
                    None => "Pick who holds it before round 1.".into(),
                },
                th().text,
            ));
        }
        if self.show_gate && w.grid.is_exit(c) {
            let note = w.hold.as_ref().map(|g| g.note(w)).unwrap_or_default();
            tip.push(if self.open { ("The gate is open.".into(), th().good) } else if note.is_empty() { (format!("The gate is shut until someone holds {}.", self.label), th().foe) } else { (note, th().foe) });
        }
        tip
    }
}

/// "Kaelen holds the winch." (ENCOUNTER2 H7's objective line; "" before the pick or without a hold escape).
pub(super) fn hold_line(s: &Session) -> String {
    match s.w.hold.as_ref().and_then(|g| g.holder.map(|h| (h, &g.label))) {
        Some((h, label)) => format!("{} holds {label}.", s.w.units[h].name),
        None => String::new(),
    }
}

/// The heroes a hold pick offers (H1: the hold escape's heroes on the field, party order).
pub(super) fn hold_pick_ids(s: &Session) -> Vec<usize> {
    s.w.hold.as_ref().map(|g| g.heroes.iter().copied().filter(|&c| s.w.has(c)).collect()).unwrap_or_default()
}

/// An exit's sign: EXIT (green) without a hold escape or with the `gate` layer off; else SHUT (red) or OPEN (green).
pub(super) fn exit_sign(hold: Option<&HoldMarks>) -> (&'static str, Rgba, Rgba) {
    let green = (rgba(70, 200, 90, 230), rgba(200, 255, 210, 255));
    match hold.filter(|h| h.show_gate) {
        Some(h) if !h.open => ("SHUT", rgba(200, 60, 56, 235), rgba(255, 210, 200, 255)),
        Some(_) => ("OPEN", green.0, green.1),
        None => ("EXIT", green.0, green.1),
    }
}

/// H7: each hold tile with a gold objective ring and a plate with the label's first letter ("W" for the winch).
pub(super) fn draw_hold_tiles(p: &mut Painter, h: &HoldMarks, geo: &Geom) {
    if !h.show_tiles {
        return;
    }
    let letter: String = h.label.trim_start_matches("the ").chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default();
    for c in &h.tiles {
        let r = match geo.view {
            View::Stage => inset2(geo.cell_rect(*c), 1, 1),
            View::Flat => geo.cell_rect(*c).inset(1),
        };
        plate(p, r, rgba(244, 198, 84, 36), rgba(244, 198, 84, 230));
        plate(p, r.inset(2), rgba(0, 0, 0, 0), rgba(244, 198, 84, 120));
        if !letter.is_empty() {
            let b = Rect::new(r.x + 2, r.y + 2, 9, 9);
            p.panel(b, rgba(60, 44, 12, 235), th().gold, 2.0);
            p.text_center(b.x as f32 + 4.5, b.y as f32 + (b.h as f32 - p.line_h(SMALL)) / 2.0, &letter, SMALL, th().gold);
        }
    }
}

/// H1: "Who holds the winch?" with a button per hero on the field (the host's `stay_pick`); over the board's lower edge
/// so the board and the hold tiles stay in view.
pub(super) fn draw_hold_pick(ui: &mut Ui<Action>, s: &Session) {
    let Some(gate) = s.w.hold.as_ref() else { return };
    let ids = hold_pick_ids(s);
    let title = format!("Who holds {}?", gate.label);
    let labels: Vec<String> = ids.iter().map(|&c| format!("{}  HP {}/{}", s.w.units[c].name, s.w.units[c].hp, s.w.units[c].max_hp)).collect();
    let widths: Vec<i32> = labels.iter().map(|l| (ui.p.text_w(l, BODY) + 12.0).ceil() as i32).collect();
    let inner = widths.iter().sum::<i32>() + 4 * (ids.len() as i32 - 1).max(0);
    let w = (inner.max((ui.p.text_w(&title, TITLE) + 12.0).ceil() as i32) + 12).min(GRID_W * TILE - 8);
    let h = 18 + ui.p.line_h(TITLE).ceil() as i32;
    let r = Rect::new(GRID_X + (GRID_W * TILE - w) / 2, GRID_Y + GRID_H * TILE - h - 6, w, h);
    ui.p.panel(r, th().game("hold_pick_fill", rgba(14, 18, 30, 240)), th().gold, RADIUS);
    ui.p.text_center(r.x as f32 + r.w as f32 / 2.0, r.y as f32 + 3.0, &title, TITLE, th().gold);
    ui.push("hold.pick", "panel", title.clone(), r, true, false, None, vec![(title.clone(), th().gold), (format!("The holder keeps {} so the gate stays open; they stay behind when the others escape.", gate.label), th().text)]);
    let mut x = r.x + (r.w - inner) / 2;
    let y = r.y + r.h - 16;
    for ((c, label), bw) in ids.iter().zip(&labels).zip(&widths) {
        let tip = vec![(format!("{} holds {}", s.w.units[*c].name, gate.label), th().gold), ("They walk to it and guard it; the gate is open while they stand on it.".into(), th().text)];
        ui.button_full(&format!("hold.pick.{c}"), Rect::new(x, y, *bw, 14), label, Action::StayPick(*c), true, false, BODY, tip);
        x += bw + 4;
    }
}

// ---- HOOKS1-UI slice 4: push and pull forecasts (HOOKS1 5) ---------------------------------------------------------

/// A push forecast's arrow.
const PUSH: Rgba = rgb(110, 228, 210);

/// HOOKS1-UI slice 4: the forced moves each of hero `h`'s planned entries would make if it struck now (aligned with its
/// compiled plan): the kit's `forced::forecast` from the tile the entry strikes from at its last block, on the unit
/// an attack targets or the units an area's cells hold. Empty without the rule forced_move.
pub(super) fn push_forecasts(s: &Session, h: usize) -> Vec<Vec<emberwake_battle::forced::Forecast>> {
    let w = &s.w;
    if !emberwake_battle::forced::on(w) {
        return Vec::new();
    }
    s.plan
        .compile(w, h)
        .entries
        .iter()
        .map(|e| {
            let cmd = e.build_command(w, h);
            let targets: Vec<usize> = match &cmd.kind {
                CmdKind::Attack { target, .. } => target.iter().copied().collect(),
                CmdKind::Area { cells, .. } => cells.iter().filter_map(|c| w.occupant(*c)).collect(),
                _ => Vec::new(),
            };
            emberwake_battle::forced::forecast(w, &s.resolver.scheduled, h, strike_tile(e), &cmd, e.end_block(w), &targets)
        })
        .collect()
}

/// A forecast in words: "Pushes Brawler 2 tiles to (7, 4); then hits the wall", "Brawler holds firm".
pub(super) fn push_text(w: &World, f: &emberwake_battle::forced::Forecast) -> (String, Rgba) {
    let name = &w.units[f.unit].name;
    if f.held {
        return (format!("{name} holds firm (no {})", if f.pull { "pull" } else { "push" }), th().dim);
    }
    let n = f.path.len();
    let verb = if f.pull { "Pulls" } else { "Pushes" };
    let mut line = if n == 0 { format!("{verb} {name} into {} at once (collision)", f.blocked_by) } else { format!("{verb} {name} {n} tile{} to {}", if n == 1 { "" } else { "s" }, f.to) };
    if n > 0 && !f.blocked_by.is_empty() {
        line += &format!("; then hits {} (collision)", f.blocked_by);
    }
    (line, if f.blocked_by.is_empty() { PUSH } else { th().foe })
}

/// The agent link's record of a forecast (tiles as [x, y]).
pub(super) fn push_json(f: &emberwake_battle::forced::Forecast) -> Value {
    let cell = |c: &V2| json!([c.x, c.y]);
    json!({"unit": f.unit, "pull": f.pull, "from": cell(&f.from), "to": cell(&f.to), "path": f.path.iter().map(cell).collect::<Vec<_>>(), "held": f.held, "blocked_by": f.blocked_by, "blocker": f.blocker})
}

/// The forecasts over the board: an arrow from the unit along its path with a head at its end; a collision's red cross
/// at the edge of the refused tile; a hold's HOLD plate.
pub(super) fn draw_pushes(p: &mut Painter, list: &[emberwake_battle::forced::Forecast], geo: &Geom) {
    for f in list {
        let (fx, fy) = ground_pt(geo, f.from);
        if f.held {
            let tw = p.text_w("HOLD", SMALL).ceil() as i32 + 6;
            let r = Rect::new(fx.round() as i32 - tw / 2, fy.round() as i32 - 5, tw, 9);
            p.panel(r, rgba(30, 70, 110, 235), HELD, 2.0);
            p.text_center(fx, r.y as f32 + (r.h as f32 - p.line_h(SMALL)) / 2.0, "HOLD", SMALL, th().text);
            continue;
        }
        let (tx, ty) = ground_pt(geo, f.to);
        if f.to != f.from {
            stage::wide_line(p, fx, fy, tx, ty, PUSH, 2.0, 0.0);
            let (dx, dy) = (tx - fx, ty - fy);
            let len = (dx * dx + dy * dy).sqrt().max(1.0);
            let (ux, uy) = (dx / len, dy / len);
            for s in [1.0f32, -1.0] {
                stage::wide_line(p, tx, ty, tx - ux * 6.0 - uy * 4.0 * s, ty - uy * 6.0 + ux * 4.0 * s, PUSH, 2.0, 0.0);
            }
        }
        if !f.blocked_by.is_empty() {
            let (nx, ny) = ground_pt(geo, f.to + f.dir);
            let (cx, cy) = ((tx + nx) / 2.0, (ty + ny) / 2.0);
            // a dark-edged cross so it reads over the tokens
            for (col, wd) in [(rgba(20, 8, 10, 230), 3.0), (th().foe, 1.6)] {
                for s in [1.0f32, -1.0] {
                    stage::wide_line(p, cx - 6.0, cy - 6.0 * s, cx + 6.0, cy + 6.0 * s, col, wd, 0.0);
                }
            }
        }
    }
}

// ---- HOOKS1-UI slice 3: unit meters (UM7), held guard (8.3) ------------------------------------------------------

/// A meter's pips and bar.
pub(super) const METER: Rgba = rgb(196, 128, 255);
/// A held guard's card edge and hover line.
pub(super) const HELD: Rgba = rgb(110, 190, 255);

/// Unit `i`'s meters while the `meters` layer is on and it is active (`meter::shown`: none without the rule unit_meters,
/// a `hidden` meter left out).
pub(super) fn meters_of(w: &World, i: usize, layers: &Layers) -> Vec<emberwake_battle::meter::Shown> {
    if !layers.meters || !w.units[i].is_active() {
        return Vec::new();
    }
    emberwake_battle::meter::shown(w, i)
}

/// "Fury 3/5 (Hot)": a meter's name, value, max and band.
pub(super) fn meter_text(m: &emberwake_battle::meter::Shown) -> String {
    let band = if m.band.is_empty() { String::new() } else { format!(" ({})", m.band) };
    format!("{} {}/{}{band}", m.name, m.value, m.max)
}

/// Up to two meters side by side in `r`: pips (one per step from min to max, the value's filled) or a bar.
pub(super) fn meter_row(p: &mut Painter, r: Rect, ms: &[emberwake_battle::meter::Shown]) {
    let n = ms.len().min(2) as i32;
    if n == 0 {
        return;
    }
    let seg = (r.w - (n - 1) * 4) / n;
    for (k, m) in ms.iter().take(2).enumerate() {
        let x = r.x + k as i32 * (seg + 4);
        let steps = (m.max - m.min).max(1);
        let filled = (m.value - m.min).clamp(0, steps);
        if m.show == "pips" {
            let pw = ((seg - (steps - 1)) / steps).max(1);
            for j in 0..steps {
                p.fill(Rect::new(x + j * (pw + 1), r.y, pw, r.h), if j < filled { METER } else { rgb(44, 36, 62) });
            }
        } else {
            p.fill(Rect::new(x, r.y, seg, r.h), rgb(44, 36, 62));
            p.fill(Rect::new(x, r.y, seg * filled / steps, r.h), METER);
        }
    }
}

/// HOOKS1 8.3: unit `i`'s held guard now (`trigger::held_at` at the session's block; None without the rule triggers,
/// for a unit not on the field, or once it ends).
pub(super) fn held_guard(s: &Session, i: usize) -> Option<emberwake_rules::unit::HeldGuard> {
    let w = &s.w;
    if !emberwake_battle::trigger::on(w) || !w.units[i].is_active() {
        return None;
    }
    emberwake_battle::trigger::held_at(w, i, s.current_block)?;
    w.units[i].held_guard
}

/// The HP cost of the skill a popup step names (the host's `skills` reply).
pub(super) fn skills_hp(skills: &[Value], step: &Value) -> Option<i64> {
    skills.iter().find(|k| k["skill"] == step["skill"]).and_then(|k| k["hp_cost"].as_i64()).filter(|c| *c > 0)
}

// ---- RULESET1-U slice 5: poise, breaks, the strip's parts, forecasts, the round summary ----------------------------

/// Unit `i`'s poise bar while its layer is on and it has one (rule poise; never in classic).
pub(super) fn poise_shown(w: &World, i: usize, layers: &Layers) -> Option<emberwake_battle::poise::PoiseState> {
    emberwake_battle::poise::state(w, i).filter(|st| layers.poise && st.max > 0 && w.units[i].is_active()).copied()
}

/// VISCLAR1-D (decision "Battle view answers, part 2", POISE1's look, no rule in it): poise shows as this many pips, each
/// 20 % of the unit's Max Poise.
pub(super) const POISE_PIPS: i32 = 5;

/// The pips lit for `value` of `max`: every started 20 % lights one, so none lit means no poise left; a windup bonus over
/// the max stays at all five; none once broken this round.
pub(super) fn poise_pips(value: i32, max: i32, broken: bool) -> i32 {
    if broken || max <= 0 {
        return 0;
    }
    (value.clamp(0, max) * POISE_PIPS + max - 1) / max
}

/// Pip `k`'s rect in row `r`: relative to the row, so a card's and a token's are the same five with a 1-unit gap.
pub(super) fn poise_pip_rect(r: Rect, k: i32) -> Rect {
    let x = |k: i32| r.x + k * (r.w + 1) / POISE_PIPS;
    Rect::new(x(k), r.y, x(k + 1) - x(k) - 1, r.h)
}

/// A poise row in `r`: the pips lit in `POISE` (a windup bonus over the max shows as a gold tip on the last); all five on a
/// red ground once broken this round.
pub(super) fn poise_bar(p: &mut Painter, r: Rect, st: &emberwake_battle::poise::PoiseState) {
    let lit = poise_pips(st.value, st.max, st.broken);
    for k in 0..POISE_PIPS {
        let pip = poise_pip_rect(r, k);
        p.fill(pip, if st.broken { rgba(120, 30, 34, 255) } else if k < lit { POISE } else { rgb(28, 34, 48) });
    }
    if !st.broken && st.value > st.max {
        p.fill(Rect::new(r.x + r.w - 2, r.y, 2, r.h), th().gold);
    }
}

/// The units broken this round (P7), while the breaks layer is on.
pub(super) fn broken_units(w: &World, layers: &Layers) -> Vec<usize> {
    if !layers.breaks {
        return Vec::new();
    }
    (0..w.units.len()).filter(|&i| w.units[i].is_active() && w.has(i) && emberwake_battle::poise::state(w, i).is_some_and(|st| st.broken)).collect()
}

/// The BREAK plate centred on x with its top at y (air layer, over the tokens).
pub(super) fn break_plate(p: &mut Painter, x: f32, y: f32) {
    let tw = p.text_w("BREAK", SMALL).ceil() as i32 + 6;
    let r = Rect::new(x.round() as i32 - tw / 2, y.round() as i32, tw, 9);
    p.panel(r, rgba(150, 36, 40, 235), rgba(255, 220, 210, 230), 2.0);
    p.text_center(x, r.y as f32 + (r.h as f32 - p.line_h(SMALL)) / 2.0, "BREAK", SMALL, th().text);
}

/// RULESET1-U slice 5, P7: once the round is confirmed, whether a break cancelled each of the selected hero's strip spans
/// (aligned with `entries`; VISCLAR1-C: the strip draws no parts, so a planned span needs no reading).
pub(super) fn broken_spans(s: &Session, u: usize, entries: &[(i32, i32, String, String, Option<String>, i32, i32)]) -> Vec<bool> {
    entries.iter().map(|e| s.resolver.scheduled.iter().any(|a| a.unit == u && a.start == e.0 && a.command.name == e.2 && a.broken)).collect()
}

/// One planned hit's forecast (RULESET1 12 "forecasts").
pub(super) struct HitForecast {
    pub(super) target: usize,
    skill: String,
    block: i32,
    stagger: i32,
    pub(super) fc: emberwake_battle::poise::Forecast,
    flank: bool,
    pierce: bool,
}

impl HitForecast {
    /// The strip tip's lines: the stagger and its factors, the verdict, flank and pierce.
    pub(super) fn lines(&self, w: &World) -> Vec<(String, Rgba)> {
        let f = &self.fc.factors;
        let mut why = String::new();
        for (on, name, m) in [(f.windup != 1.0, "in wind-up", f.windup), (f.guard != 1.0, "guarding", f.guard), (f.repeat != 1.0, "broke last round", f.repeat), (f.flank != 1.0, "flank", f.flank), (f.passive != 1.0, "trait", f.passive), (f.hook != 1.0, "hook", f.hook)] {
            if on {
                why += &format!(" x {name} {m}");
            }
        }
        let mut out = vec![(format!("Forecast on {}", w.units[self.target].name), th().gold)];
        if self.staggers() {
            out.push((format!("Stagger {}{why} = {}", self.stagger, self.fc.taken), th().text));
            out.push(self.verdict());
        }
        if self.flank {
            out.push(("Flank".into(), th().good));
        }
        if self.pierce {
            out.push(("Armour pierced".into(), th().good));
        }
        out
    }

    /// The verdict line ("Breaks: poise 20 -> 0", "May break: 40%", "Poise 40 -> 28").
    fn verdict(&self) -> (String, Rgba) {
        use emberwake_battle::poise::{BreakCause, Verdict};
        let (b, a) = (self.fc.before, self.fc.after);
        match self.fc.verdict {
            Verdict::Breaks(BreakCause::Sure) => ("Breaks: a sure break into its wind-up".into(), th().foe),
            Verdict::Breaks(_) => (format!("Breaks: poise {b} -> 0"), th().foe),
            Verdict::Chance(c) => (format!("May break: {c}% (poise {b} -> {a})"), th().gold),
            Verdict::Holds => (format!("Poise {b} -> {a}"), th().text),
            Verdict::AlreadyBroken => ("Already broken this round".into(), th().dim),
        }
    }

    /// Whether the hit touches poise at all (stagger, or a sure break).
    fn staggers(&self) -> bool {
        self.stagger > 0 || matches!(self.fc.verdict, emberwake_battle::poise::Verdict::Breaks(emberwake_battle::poise::BreakCause::Sure))
    }

    /// One intel panel line: "Your Jab B3: in wind-up, breaks: poise 20 -> 0, flank" style.
    pub(super) fn short(&self, _w: &World) -> (String, Rgba) {
        let mut bits: Vec<String> = Vec::new();
        let mut col = th().text;
        if self.staggers() {
            let (v, c) = self.verdict();
            col = c;
            if self.fc.windup {
                bits.push("in wind-up".into());
            }
            bits.push(v.to_lowercase());
        }
        if self.flank {
            bits.push("flank".into());
        }
        if self.pierce {
            bits.push("armour pierced".into());
        }
        (format!("Your {} B{}: {}", self.skill, self.block, bits.join(", ")), col)
    }
}

/// RULESET1-U slice 5: the forecast of each of hero `h`'s planned entries (aligned with its compiled plan; None for
/// anything but an attack on a unit with a poise bar): the kit's `poise::forecast` against the enemies' scheduled
/// actions, on the bar the hero's earlier planned hits leave (one break a round, P8), flank from the tile the hit
/// strikes from (`round_intent::flank_bonus_from`). Empty without the rule poise.
pub(super) fn hit_forecasts(s: &Session, h: usize) -> Vec<Option<HitForecast>> {
    use emberwake_battle::poise;
    let w = &s.w;
    if !poise::on(w) {
        return Vec::new();
    }
    // the planned attacks on a unit, in plan order, forecast as a sequence by the kit (each on the bar the earlier
    // ones leave: slice 6)
    let st = s.plan.compile(w, h);
    let planned: Vec<Option<(usize, emberwake_battle::command::Command, i32, bool)>> = st
        .entries
        .iter()
        .map(|e| {
            let t = e.target.filter(|&t| t != h)?;
            if emberwake_battle::skill::kind(&w.cat.skills[e.skill]) != Kind::Attack {
                return None;
            }
            Some((t, e.build_command(w, h), e.end_block(w), ri::flank_bonus_from(w, h, strike_tile(e), t)))
        })
        .collect();
    let hits: Vec<poise::PlannedHit> = planned.iter().flatten().map(|(t, cmd, block, flank)| poise::PlannedHit { target: *t, block: *block, impact: cmd.impact, blocks: cmd.block_cost, flank: *flank, ignore_guard: cmd.ignore_guard }).collect();
    let mut fcs = poise::forecast_hits(w, &s.resolver.scheduled, &hits).into_iter();
    st.entries
        .iter()
        .zip(&planned)
        .map(|(e, p)| {
            let (t, cmd, block, flank) = p.as_ref()?;
            let fc = fcs.next()??;
            let f = HitForecast { target: *t, skill: w.cat.skills[e.skill].name.clone(), block: *block, stagger: cmd.impact.stagger, fc, flank: *flank, pierce: cmd.pierces(w) };
            // a hit with nothing to say (no stagger, no flank, no pierce) has no forecast
            (f.staggers() || f.flank || f.pierce).then_some(f)
        })
        .collect()
}

/// RULESET1 12: the round summary in words ("1 break (1 of 2 hits into a wind-up), 1 flank, 3 follow-ups, Rest +1
/// AP"; "no breaks" alone when nothing happened); None without the round intent or with the summary layer off.
pub(super) fn round_summary_text(s: &Session, layers: &Layers) -> Option<String> {
    if !ri::on(&s.w) || !layers.summary {
        return None;
    }
    Some(ri::summary_text(&ri::round_summary(&s.w, &s.resolver.scheduled)))
}
