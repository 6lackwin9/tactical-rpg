//! The board overlays under the round intent (RULESET1-U slice 3) and the overlay groups behind their flags (`OVERLAY_GROUPS`, `Layers`).

use aegis_ui::theme::th;
use super::board::{inset2, plate, with_alpha};
use super::marks::{arrows_json, Arrow};
use super::{AMBER, FACING};
use aegis_ui::canvas::rgba;
use crate::stage::{self, Geom, View};
use aegis_ui::painter::Painter;
use emberwake_battle::grid::{V2, GRID_H, GRID_W};
use emberwake_battle::round_intent::{self as ri, RoundIntent, RoundTarget};
use emberwake_battle::session::Session;
use emberwake_battle::skill::Kind;
use emberwake_battle::world::World;
use serde_json::{json, Value};

// ---- RULESET1-U slice 3: the board overlays under the round intent (RULESET1_INTENT UI3, RULESET1 K6) -------------

/// How far around the round target the Maneuver front is shaded (tiles, Manhattan).
pub(super) const FRONT_RADIUS: i32 = 3;

/// A unit's "Reach" on its hover card and the hero panel: under the rule `reach` its plain attack band ("1-2", K6:
/// kit `skill::attack_band`), else its `attack_range` (classic, as before).
pub(super) fn reach_text(w: &World, i: usize) -> String {
    if !w.ruleset.has(emberwake_rules::ruleset::REACH) {
        return w.units[i].attack_range.to_string();
    }
    match emberwake_battle::skill::attack_band(&w.units[i], &w.ruleset) {
        (lo, hi) if lo == hi => lo.to_string(),
        (lo, hi) => format!("{lo}-{hi}"),
    }
}

/// How a planned move of the selected hero is drawn: a plain move before its commands (walk), a move its skill
/// carries (MV4: an Engage's dash, a chase), or a step after its commands (AS3's kite step, MV8's retreat).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MoveKind {
    Walk,
    Carried,
    After,
}

impl MoveKind {
    fn id(self) -> &'static str {
        match self {
            MoveKind::Walk => "walk",
            MoveKind::Carried => "carried",
            MoveKind::After => "after",
        }
    }
}

/// RULESET1-U slice 3: what the board draws under the round intent, every piece a kit query (none in classic).
#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct Overlay {
    /// The selected hero's round target tile (a unit's or the target tile; None for self or none). Planning only.
    pub(super) target: Option<V2>,
    /// The tiles of the units a board click would make the hero's round target (the intent table, PL2; UI1 "valid
    /// cells lit"), the current target left out. Planning only.
    pub(super) valid: Vec<V2>,
    /// K6: the band of the hero's plain attack (`skill::attack_band`) around the tile its first attack strikes from
    /// (else where it stands), under Assault and Maneuver. Planning only.
    pub(super) band: Vec<V2>,
    /// UI3: under Maneuver, the target's front within `FRONT_RADIUS` (`round_intent::in_front`: no flank there).
    pub(super) front: Vec<V2>,
    /// The hero's planned moves in order: the tiles each passes (from first) and how it is drawn.
    pub(super) moves: Vec<(Vec<V2>, MoveKind)>,
    /// MV1: every active enemy's tile and facing (a sign vector; units without one left out). Planning and playback.
    pub(super) facings: Vec<(V2, V2)>,
    /// GD5: the tiles of stealthed units, and whether each is hidden now (moved this round). Planning and playback.
    pub(super) veils: Vec<(V2, bool)>,
    /// VISCLAR1 AR1-AR6 (they replace RULESET1-U's tell arrows, planned-move lines and revealed enemy walks): every
    /// unit's one target arrow (`arrows_of`; the screen fills it, `shown` keeps the selected unit's and the inspected
    /// unit's, or all while Alt is held). Planning only.
    pub(super) arrows: Vec<Arrow>,
    /// INPUT1 MR13: while a Maneuver Engage row of the skill menu is hovered, its target's open position tiles (true:
    /// the preferred tier); none when the flank is protected. Planning only, layer `moves`.
    pub(super) positions: Vec<(V2, bool)>,
}

/// RULESET1-U slice 6: the overlay groups as the Settings Battle tab lists them: (id, label, what it shows, default);
/// VISCLAR1 VC6: every group on by default but `all_bars`, `test_hazards` and `test_arena`.
pub(crate) const OVERLAY_GROUPS: [(&str, &str, &str, bool); 24] = [
    ("band", "Reach band", "The selected hero's attack reach around the tile it strikes from (Assault, Maneuver).", true),
    ("front", "Maneuver front", "The target's front, where an Engage never stands (Maneuver).", true),
    ("valid", "Valid targets", "Rings on the units a click would make the round target.", true),
    ("veils", "Stealth veils", "A veil under each stealthed unit, filled once it is hidden.", true),
    // VISCLAR1 AR5: the moves now draw as destination plates and the move leg of the party's target arrows
    ("moves", "Planned moves", "A plate where each of the selected hero's moves ends, and the move leg of the party's target arrows (off: straight).", true),
    ("target", "Target ring", "The ring on the selected hero's round target.", true),
    ("facings", "Enemy facings", "A chevron where each enemy faces (a flank comes from beside or behind).", true),
    // VISCLAR1 AR5: a revealed enemy's moves are its target arrow's move leg
    ("walks", "Revealed enemy moves", "The move leg of a revealed enemy's target arrow (off: the arrow goes straight to its target).", true),
    ("glyphs", "Intent glyphs", "The round intent's plate on the strip, the cards and the tokens.", true),
    ("badges", "Impact badges", "The row of blocks on each hostile card with its impacts.", true),
    ("poise", "Poise bars", "The poise bars under the HP bars.", true),
    // VISCLAR1 VC3: today's look, every unit's bars on the board
    ("all_bars", "All bars", "Every unit's HP and poise bars on the board; off: only the selected hero, its target and the unit under the mouse (hold Alt: all).", false),
    ("breaks", "Breaks", "BREAK over a broken unit and on its cancelled strip span.", true),
    ("parts", "Wind-up and impact", "The full timeline's wind-up, impact and recovery shading.", true),
    ("forecasts", "Hit forecasts", "Stagger and break forecasts in the strip tips and the intel panel.", true),
    ("summary", "Round summary", "The round summary in the message bar at round end.", true),
    // HOOKS1-UI slice 1: the hooks' marks (HOOKS1 GH6, AH3). VISCLAR1-B, decision "Battle view answers": the board's marks
    // for testing (in the real game they are part of the arena), off by default; Alt shows them and the hover lines
    // always tell. New ids, as every settings file already
    // saved holds `hazards` and `arena` on, which would keep them on.
    ("test_hazards", "Ground hazards (testing)", "Testing: the hazards skills and items leave, drawn on their tiles in their tint, with their glyph (a foe's hidden one stays hidden). Off: hold Alt to draw them; pointing at the tile always tells.", false),
    ("test_arena", "Arena telegraph (testing)", "Testing: the cells an arena hazard strikes this round drawn on the board, and a badge on the strip at the blocks it strikes. Off: hold Alt to draw them; pointing at the tile or block always tells.", false),
    // HOOKS1-UI slice 3: unit meters (UM7), held guard (8.3)
    ("meters", "Unit meters", "Each unit's meters as pips or a bar on its card, and the selected hero's under the strip title.", true),
    ("guard", "Held guard", "A blue edge on the card of a unit holding a guard a trigger gave it (it counts as guarding).", true),
    // HOOKS1-UI slice 4: push and pull forecasts (HOOKS1 5)
    ("pushes", "Push forecasts", "Where the selected hero's planned pushes and pulls would move their targets, and what they would collide with.", true),
    // ENCOUNTER2-U: a hold escape's marks (ENCOUNTER2 H7)
    ("hold", "Hold tiles", "The tiles a hold escape's holder keeps, with the holder's name on their card.", true),
    ("gate", "Gate state", "A hold escape's exits read SHUT (red) until the holder stands on a hold tile, then OPEN (green).", true),
    // INPUT1 PB1: the playback's impact tracers
    ("tracers", "Impact tracers", "A short line from each actor to what its action hit, during playback.", true),
];

/// VISCLAR1 VC6: a group's default (`OVERLAY_GROUPS`; an unknown id is on).
pub(crate) fn overlay_default(id: &str) -> bool {
    OVERLAY_GROUPS.iter().find(|g| g.0 == id).is_none_or(|g| g.3)
}

/// RULESET1-U slice 5 (preference "Battle overlays and the clarity pass", user 2026-10-30): every timeline overlay
/// group behind its own flag, on by default (they help debugging the fight); the settings key `battle_overlays`
/// ({<group>: false}) switches a group off; slice 6 lists them on the Settings Battle tab; VISCLAR1 slice A adds
/// `all_bars`, `test_hazards` and `test_arena`, off by default (VC6). Classic draws none of them whatever the flags say.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Layers {
    /// Slice 3: the reach band, the Maneuver front, the valid-target rings, the stealth veils, the hero's moves, the
    /// target ring, the enemy facings.
    pub(super) band: bool,
    pub(super) front: bool,
    valid: bool,
    veils: bool,
    pub(super) moves: bool,
    target: bool,
    pub(super) facings: bool,
    /// Slices 4-5: revealed enemy moves, intent glyphs, impact badge rows.
    pub(super) walks: bool,
    pub(super) glyphs: bool,
    pub(crate) badges: bool,
    /// Slice 5: poise bars, BREAK markers, impact / recovery on the strip (the full timeline adds wind-up), hit forecasts, the round summary.
    pub(super) poise: bool,
    /// VISCLAR1 VC3: every unit's token bars, not only the focus set's (off by default).
    pub(super) all_bars: bool,
    pub(crate) breaks: bool,
    pub(crate) parts: bool,
    pub(super) forecasts: bool,
    pub(super) summary: bool,
    /// HOOKS1-UI slice 1: ground hazards (GH6), the arena telegraph (AH3).
    pub(super) hazards: bool,
    pub(super) arena: bool,
    /// HOOKS1-UI slice 3: unit meters (UM7), held guard (8.3).
    pub(super) meters: bool,
    pub(super) guard: bool,
    /// HOOKS1-UI slice 4: push and pull forecasts (HOOKS1 5).
    pub(super) pushes: bool,
    /// ENCOUNTER2-U: a hold escape's tiles and gate (H7).
    pub(super) hold: bool,
    pub(super) gate: bool,
    /// INPUT1 PB1: the playback's impact tracers.
    pub(super) tracers: bool,
}

impl Layers {
    /// The group ids, as the settings key names them (`OVERLAY_GROUPS`' order).
    pub(super) const IDS: [&'static str; 24] = {
        let mut ids = [""; 24];
        let mut k = 0;
        while k < 24 {
            ids[k] = OVERLAY_GROUPS[k].0;
            k += 1;
        }
        ids
    };

    /// The flags of the settings key `battle_overlays` (a missing group, or no key, takes its default: VC6).
    pub(crate) fn of(v: &Value) -> Layers {
        let on = |id: &str| v.get(id).and_then(Value::as_bool).unwrap_or_else(|| overlay_default(id));
        Layers {
            band: on("band"),
            front: on("front"),
            valid: on("valid"),
            veils: on("veils"),
            moves: on("moves"),
            target: on("target"),
            facings: on("facings"),
            walks: on("walks"),
            glyphs: on("glyphs"),
            badges: on("badges"),
            poise: on("poise"),
            all_bars: on("all_bars"),
            breaks: on("breaks"),
            parts: on("parts"),
            forecasts: on("forecasts"),
            summary: on("summary"),
            hazards: on("test_hazards"),
            arena: on("test_arena"),
            meters: on("meters"),
            guard: on("guard"),
            pushes: on("pushes"),
            hold: on("hold"),
            gate: on("gate"),
            tracers: on("tracers"),
        }
    }

    /// The flags by group id (the agent link's state).
    pub(super) fn json(&self) -> Value {
        let v = [self.band, self.front, self.valid, self.veils, self.moves, self.target, self.facings, self.walks, self.glyphs, self.badges, self.poise, self.all_bars, self.breaks, self.parts, self.forecasts, self.summary, self.hazards, self.arena, self.meters, self.guard, self.pushes, self.hold, self.gate, self.tracers];
        Value::Object(Self::IDS.iter().zip(v).map(|(k, b)| (k.to_string(), json!(b))).collect())
    }
}

impl Overlay {
    /// What the board draws of it: the groups whose layer is on; AR3: the target arrows of the selected unit `sel` and
    /// the inspected unit `inspect` (decision "INPUT1 answers, part 4" Q15), every unit's while `alt` is held.
    pub(super) fn shown(&self, l: &Layers, sel: Option<usize>, inspect: Option<usize>, alt: bool) -> Overlay {
        let keep = |on: bool, v: &Vec<V2>| if on { v.clone() } else { Vec::new() };
        Overlay {
            target: self.target.filter(|_| l.target),
            valid: keep(l.valid, &self.valid),
            band: keep(l.band, &self.band),
            front: keep(l.front, &self.front),
            moves: if l.moves { self.moves.clone() } else { Vec::new() },
            facings: if l.facings { self.facings.clone() } else { Vec::new() },
            veils: if l.veils { self.veils.clone() } else { Vec::new() },
            arrows: self.arrows.iter().filter(|a| alt || Some(a.unit) == sel || Some(a.unit) == inspect).cloned().collect(),
            positions: if l.moves { self.positions.clone() } else { Vec::new() },
        }
    }
}

/// The overlays of the battle now: None without the rule `round_intent` (classic draws none).
pub(super) fn overlay_of(s: &Session, sel: Option<usize>, planning: bool) -> Option<Overlay> {
    let w = &s.w;
    if !ri::on(w) {
        return None;
    }
    let on_field = |u: usize| w.units[u].is_active() && w.has(u);
    let mut o = Overlay {
        facings: w.enemies.iter().copied().filter(|&e| on_field(e)).map(|e| (w.pos(e), ri::facing(w, e))).filter(|(_, f)| *f != emberwake_battle::grid::ZERO).collect(),
        veils: (0..w.units.len()).filter(|&u| on_field(u) && w.units[u].has_status("stealth")).map(|u| (w.pos(u), ri::hidden(w, u))).collect(),
        ..Overlay::default()
    };
    let Some(h) = sel.filter(|&h| planning && on_field(h) && w.allies.contains(&h)) else { return Some(o) };
    let target = w.round_states.get(&h).and_then(|r| r.target);
    o.target = match target {
        Some(RoundTarget::Unit(t)) if on_field(t) => Some(w.pos(t)),
        Some(RoundTarget::Cell(p)) => Some(p),
        _ => None,
    };
    let st = s.plan.compile(w, h);
    // the planned moves: a carried move rides in its skill's entry (Engage, a chase), a retreat or a move after the
    // round's first command is an after-move
    let mut from = w.pos(h);
    let mut acted = false;
    for e in &st.entries {
        let kind = emberwake_battle::skill::kind(&w.cat.skills[e.skill]);
        if e.pos_after != from && e.pos_after.x >= 0 {
            // the entry's path is its steps (direction vectors) from where it starts (the slice 3 review: the line
            // starts there too, not at the last drawn tile)
            let start = if w.grid.is_valid(e.pos_before) { e.pos_before } else { from };
            let mut tiles = vec![start];
            let mut at = start;
            for d in &e.path {
                at = at + *d;
                if tiles.last() != Some(&at) && w.grid.is_valid(at) {
                    tiles.push(at);
                }
            }
            if tiles.last() != Some(&e.pos_after) {
                tiles.push(e.pos_after);
            }
            let how = match (&e.carry, kind) {
                (emberwake_battle::command::Carry::Retreat(_), _) => MoveKind::After,
                (_, Kind::Move) if acted => MoveKind::After,
                (_, Kind::Move) => MoveKind::Walk,
                _ => MoveKind::Carried,
            };
            o.moves.push((tiles, how));
            from = e.pos_after;
        }
        if !matches!(kind, Kind::Move | Kind::Rest) {
            acted = true;
        }
    }
    let Some(intent) = ri::intent_of(w, h) else { return Some(o) };
    o.valid = (0..w.units.len())
        .filter(|&u| on_field(u) && Some(ri::normalize(h, RoundTarget::Unit(u))) != target)
        .filter(|&u| ri::target_problem(w, h, intent, RoundTarget::Unit(u)).is_empty())
        .map(|u| w.pos(u))
        .collect();
    let all_cells = || (0..GRID_H).flat_map(|y| (0..GRID_W).map(move |x| V2::new(x, y))).filter(|c| w.grid.is_valid(*c));
    if matches!(intent, RoundIntent::Assault | RoundIntent::Maneuver) {
        let (lo, hi) = emberwake_battle::skill::attack_band(&w.units[h], &w.ruleset);
        let at = st.entries.iter().find(|e| emberwake_battle::skill::kind(&w.cat.skills[e.skill]) == Kind::Attack).map(strike_tile).unwrap_or_else(|| w.pos(h));
        o.band = all_cells().filter(|c| (lo..=hi).contains(&c.dist(at))).collect();
    }
    if let (RoundIntent::Maneuver, Some(RoundTarget::Unit(t))) = (intent, target) {
        if on_field(t) {
            let tp = w.pos(t);
            o.front = all_cells().filter(|c| c.dist(tp) <= FRONT_RADIUS && *c != tp && ri::in_front(w, t, *c)).collect();
        }
    }
    Some(o)
}

/// The tile an attack entry strikes from: after its carried Engage move (MV4: the move comes before the hit), before
/// its Disengage retreat (MV8: the retreat comes after the hit).
pub(super) fn strike_tile(e: &emberwake_battle::plan::PlanEntry) -> V2 {
    if matches!(e.carry, emberwake_battle::command::Carry::Retreat(_)) {
        e.pos_before
    } else {
        e.pos_after
    }
}

/// The overlays for the agent link's state: tiles as [x, y].
pub(super) fn overlay_json(o: &Overlay) -> Value {
    let cell = |c: &V2| json!([c.x, c.y]);
    json!({
        "target": o.target.as_ref().map(cell),
        "valid": o.valid.iter().map(cell).collect::<Vec<_>>(),
        "band": o.band.iter().map(cell).collect::<Vec<_>>(),
        "front": o.front.iter().map(cell).collect::<Vec<_>>(),
        "moves": o.moves.iter().map(|(t, k)| json!({"kind": k.id(), "tiles": t.iter().map(cell).collect::<Vec<_>>()})).collect::<Vec<_>>(),
        "facings": o.facings.iter().map(|(c, f)| json!({"at": cell(c), "facing": [f.x, f.y]})).collect::<Vec<_>>(),
        "veils": o.veils.iter().map(|(c, hidden)| json!({"at": cell(c), "hidden": hidden})).collect::<Vec<_>>(),
        "arrows": arrows_json(&o.arrows),
        "positions": o.positions.iter().map(|(c, preferred)| json!({"at": cell(c), "preferred": preferred})).collect::<Vec<_>>(),
    })
}

/// The ground point of a cell in the view: the foot point (Stage), the cell's lower part (Flat, where the token's
/// foot ellipse sits).
pub(super) fn ground_pt(geo: &Geom, c: V2) -> (f32, f32) {
    let (x, y) = geo.foot(c);
    match geo.view {
        View::Stage => (x as f32, y as f32),
        View::Flat => (x as f32, (geo.cell_rect(c).y + geo.cell_rect(c).h) as f32 - 4.0),
    }
}

/// RULESET1-U slice 3: draws the overlays `o` (as `Overlay::shown` keeps them) on the ground, under the tokens (Flat:
/// after the tiles; Stage: the markers pass), in the order back to front: the reach band, the Maneuver front, the valid
/// targets, the stealth veils, the moves' destination plates (VISCLAR1 AR5: the lines give way to the target arrows,
/// `draw_arrows`; Q14: an after-move shows as its plate only), the target ring, the enemy facings.
pub(super) fn draw_overlay(p: &mut Painter, o: &Overlay, geo: &Geom) {
    let plate_of = |c: V2| match geo.view {
        View::Stage => inset2(geo.cell_rect(c), 1, 1),
        View::Flat => geo.cell_rect(c).inset(1),
    };
    for c in &o.band {
        plate(p, plate_of(*c), rgba(110, 170, 255, 22), rgba(110, 170, 255, 70));
    }
    for c in &o.front {
        plate(p, plate_of(*c), rgba(240, 100, 92, 46), rgba(240, 100, 92, 110));
    }
    // a ground ellipse's radii in the view (the Stage squashes it like the foot rings)
    let (rx, ry) = match geo.view {
        View::Stage => (0.36 * geo.token_px as f32, 4.2),
        View::Flat => (13.0, 5.0),
    };
    for c in &o.valid {
        let (x, y) = ground_pt(geo, *c);
        p.ellipse_ring(x, y, rx + 1.0, ry + 0.8, rgba(255, 255, 255, 150));
    }
    for (c, hidden) in &o.veils {
        let (x, y) = ground_pt(geo, *c);
        p.ellipse(x, y, rx, ry, rgba(186, 170, 255, if *hidden { 90 } else { 40 }));
        p.ellipse_ring(x, y, rx, ry, rgba(186, 170, 255, if *hidden { 230 } else { 150 }));
    }
    for (tiles, how) in &o.moves {
        let col = if *how == MoveKind::Carried { AMBER } else { th().gold };
        if let Some(end) = tiles.last() {
            let r = match geo.view {
                View::Stage => inset2(geo.cell_rect(*end), 3, 2),
                View::Flat => geo.cell_rect(*end).inset(3),
            };
            p.panel(r, with_alpha(col, 40), col, 2.0);
        }
    }
    // INPUT1 MR13: the hovered Engage row's open position tiles: a GOLD ring plate (preferred), a DIM one (fallback)
    for (c, preferred) in &o.positions {
        let col = if *preferred { th().gold } else { th().dim };
        plate(p, plate_of(*c), with_alpha(col, 36), col);
    }
    if let Some(c) = o.target {
        let (x, y) = ground_pt(geo, c);
        p.ellipse_ring(x, y, rx + 2.5, ry + 1.6, th().gold);
        p.ellipse_ring(x, y, rx + 3.5, ry + 2.2, with_alpha(th().gold, 150));
    }
    // a chevron on the ground ahead of each enemy, pointing where it faces (the Stage squashes the y part)
    let squash = if geo.view == View::Stage { 0.6 } else { 1.0 };
    for (c, f) in &o.facings {
        let (x, y) = ground_pt(geo, *c);
        let len = ((f.x * f.x + f.y * f.y) as f32).sqrt().max(1.0);
        let (dx, dy) = (f.x as f32 / len, f.y as f32 / len * squash);
        let (tip, base) = (rx + 8.0, rx + 3.0);
        let (px, py) = (-dy * 4.0, dx * 4.0 * squash.max(0.8));
        let (tx, ty) = (x + dx * tip, y + dy * tip);
        let (bx, by) = (x + dx * base, y + dy * base);
        // pale over a dark underlay: apart from the red target arrows and the foot rings (slice 4: red read as part
        // of the tell arrows next to the intent glyphs; VISCLAR1 VC6 keeps it on)
        for (col, width) in [(rgba(10, 8, 12, 200), 3.2), (FACING, 1.6)] {
            stage::wide_line(p, bx + px, by + py, tx, ty, col, width, 0.0);
            stage::wide_line(p, bx - px, by - py, tx, ty, col, width, 0.0);
        }
    }
}
