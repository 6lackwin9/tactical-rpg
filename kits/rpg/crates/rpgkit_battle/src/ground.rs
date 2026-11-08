//! Ground hazards (HOOKS1 6, rule `ground_hazards`; engine-only, no Godot original). A skill's or consumable's `hazard`
//! leaves a timed overlay of a `hazards` row on tiles (GH1, GH2): an Attack, Area or Support skill at its impact (step 7
//! of a landed hit, after the forced moves), a Defend at its start, an item at use; an ATTACK with no power and a
//! hazard is a placement that aims at a tile and hits nobody (GH3). One hazard a tile, the newest replacing the old; a
//! terrain hazard stays and acts first (GH4). A hazard acts on a unit it `affects` (all, or the placer's foes or
//! friends) as it enters the tile (walk, dash, carried move, jump, forced move) and at round end after the terrain's
//! roll: flat damage through guard, not armour, then its status roll; cover and move cost add to the terrain's through
//! `Grid` (GH5). A trap stops the move that springs it on its tile, `stop_moves` cancels the unit's later Walks this
//! round, and the trap is gone (GH6); a hidden hazard is unseen by the other side. `rounds` N lasts through round
//! (placed + N - 1) and fades after that round's rolls (GH7). The AI's path penalty counts a hazard that would harm the
//! unit and that its side can see (GH8). Every entry point reads the rule first and the overlay is empty in classic, so
//! classic battles never place, roll, log or record anything new.

use crate::command::CmdKind;
use crate::grid::{V2, ZERO};
use crate::plan::AI_HAZARD_PENALTY;
use crate::resolver::{Resolver, RoundEndStep};
use crate::round_intent;
use crate::skill::{self, Kind};
use crate::world::{UnitId, World};
use aegis_core::rng::GodotRng;
use rpgkit_data::{HazardDef, HazardPlace, ItemDef, SkillDef};
use rpgkit_rules::{ruleset, status};

/// Whether the battle plays by ground hazards.
pub fn on(w: &World) -> bool {
    w.ruleset.has(ruleset::GROUND_HAZARDS)
}

/// GH2: the tiles a placement covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// The skill's own area cells (a single-target skill: the anchor tile).
    Skill,
    /// The anchor tile.
    One,
    /// The anchor and its four neighbours.
    Plus,
    /// The 3 x 3 square around the anchor.
    Burst,
    /// Three tiles in a row along the heading: from the anchor (`at` target), from the tile ahead of the user (self).
    /// A3.2-K2: the heading is IR7's snapped direction (`shape_heading`).
    Line,
    /// A3.2-S1: the 8 tiles around the anchor (target) or the user (self), never the centre.
    Ring,
    /// A3.2-S1: the 3-tile front arc; A3.2-K2 (FB-106): the skill's own ARC cells (`skill::arc_cells`) drawn on the
    /// anchor (target) or the tile ahead of the user (self), so a hazard and the skill's ARC cover the same tiles.
    Arc,
}

/// The most rounds a placed hazard lasts (a row's `rounds` is clamped to it; A4.2-K1b: the Field row's `lingering`
/// bonus too).
pub const HAZARD_ROUNDS_CAP: i32 = 5;

/// GH2: a skill's or consumable's `hazard`, carried by its command (read only under the rule).
#[derive(Debug, Clone, PartialEq)]
pub struct Place {
    /// A `hazards` row id.
    pub id: String,
    pub shape: Shape,
    /// `at` self: the user's tile; else the aim tile or the target unit's tile.
    pub at_self: bool,
    /// 0 = the row's `duration`.
    pub rounds: i32,
}

impl Place {
    fn from(p: &HazardPlace, item: bool) -> Self {
        let shape = match p.shape.as_str() {
            "SKILL" if !item => Shape::Skill,
            "PLUS" => Shape::Plus,
            "BURST" => Shape::Burst,
            "LINE" => Shape::Line,
            "RING" => Shape::Ring,
            "ARC" => Shape::Arc,
            _ => Shape::One,
        };
        Self { id: p.id.clone(), shape, at_self: p.at == "self", rounds: p.rounds.clamp(0, HAZARD_ROUNDS_CAP) }
    }

    /// The skill's placement: ATTACK, DEFEND and SUPPORT skills (not an item's synthetic skill); None without the field.
    pub fn of_skill(s: &SkillDef) -> Option<Self> {
        if skill::is_item(s) || !matches!(skill::kind(s), Kind::Attack | Kind::Defend | Kind::Support) {
            return None;
        }
        s.hazard.as_ref().map(|p| Self::from(p, false))
    }

    /// A consumable's placement at use (`shape` SKILL reads as one tile).
    pub fn of_item(i: &ItemDef) -> Option<Self> {
        i.hazard.as_ref().map(|p| Self::from(p, true))
    }
}

/// One hazard on a tile: its row, who placed it, the round it was placed and the last round it lasts (GH7).
#[derive(Debug, Clone, PartialEq)]
pub struct Placed {
    pub def: HazardDef,
    pub by: UnitId,
    pub round: i32,
    pub last_round: i32,
}

impl Placed {
    /// GH1 `affects`: every unit, or the placer's foes, or its friends (the placer included).
    pub fn affects(&self, w: &World, u: UnitId) -> bool {
        match self.def.affects.as_str() {
            "foes" => w.foes_of(self.by).contains(&u),
            "friends" => w.friends_of(self.by).contains(&u),
            _ => true,
        }
    }

    /// GH6: a hidden hazard is seen only by its placer's side.
    pub fn seen_by(&self, w: &World, u: UnitId) -> bool {
        !self.def.hidden || w.friends_of(self.by).contains(&u)
    }

    /// GH8: it can hurt a unit it affects (damage, a trap or a debuff; smoke only hides).
    pub fn harmful(&self) -> bool {
        self.def.damage > 0 || self.def.trap || (!self.def.status.is_empty() && status::debuffs().contains(&self.def.status.as_str()))
    }

    fn acts_on_enter(&self) -> bool {
        self.def.trap || self.def.on == "enter" || self.def.on == "both"
    }

    fn acts_at_round_end(&self) -> bool {
        self.def.on == "round_end" || self.def.on == "both"
    }
}

/// A placement: who left which hazard (its name and id) with which command, on which tiles, and its log line.
#[derive(Debug, Clone, PartialEq)]
pub struct Placement {
    pub by: UnitId,
    pub hazard: String,
    pub id: String,
    pub skill: String,
    pub cells: Vec<V2>,
    pub rounds: i32,
    pub line: String,
}

/// A hazard acting on a unit: the unit, the placer, the hazard (name, id), the HP lost, the status that landed ("" =
/// none), whether it was a sprung trap, a round-end roll, a `stop_moves` and how many Walks that cancelled, the log
/// line, and (on enter) the HP journal rows held back so the metrics hear them with the hazard's own cell.
#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    pub unit: UnitId,
    pub by: UnitId,
    pub hazard: String,
    pub id: String,
    pub damage: i32,
    pub status: String,
    pub trap: bool,
    pub round_end: bool,
    pub stop: bool,
    pub stopped: i32,
    pub line: String,
    pub journal: Vec<(UnitId, i32)>,
}

/// GH7: a placement's tiles faded at round end (one per hazard and placer).
#[derive(Debug, Clone, PartialEq)]
pub struct Fade {
    pub by: UnitId,
    pub hazard: String,
    pub id: String,
    pub cells: i32,
    pub line: String,
}

/// What the ground did, in order, until the session logs it.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Placed(Placement),
    Hit(Hit),
    Faded(Fade),
}

/// The battle's ground record: the events the session has not logged yet, and the hits whose `stop_moves` the resolver
/// has not applied yet (indices into `events`).
#[derive(Debug, Clone, Default)]
pub struct Ledger {
    pub events: Vec<Event>,
    stops: Vec<usize>,
}

impl Ledger {
    /// The events since the last call (the session logs them).
    pub fn take(&mut self) -> Vec<Event> {
        self.stops.clear();
        std::mem::take(&mut self.events)
    }
}

// ---- Placement (GH2-GH4, GH7) --------------------------------------------------------------------------------------

/// GH2 (LINE / ARC with no aim, `shape_heading`): the user's heading toward `aim`: the dominant axis (a tie on its facing axis, else x); with no aim
/// off its own tile, its straight facing, else toward the other side.
fn heading(w: &World, by: UnitId, aim: V2) -> V2 {
    let d = crate::forced::direction(w, by, w.pos(by), aim, false);
    if d != ZERO {
        return d;
    }
    let f = round_intent::facing(w, by);
    if f.x != 0 {
        V2::new(f.x, 0)
    } else if f.y != 0 {
        V2::new(0, f.y)
    } else if w.allies.contains(&by) {
        V2::new(1, 0)
    } else {
        V2::new(-1, 0)
    }
}

/// A3.2-K2 (INTENT2 IR7): the direction of a LINE or ARC placement: from the user toward `aim`, snapped
/// (`skill::snap`: a tie is the exact diagonal); with no aim off its own tile, `heading`'s fallback (its facing).
fn shape_heading(w: &World, by: UnitId, aim: V2) -> V2 {
    let d = skill::snap(aim - w.pos(by));
    if d != ZERO {
        d
    } else {
        heading(w, by, aim)
    }
}

/// GH2: the tiles `p` covers from `by` (anchor: the aim or target tile, or the user's; `area`: the skill's own cells):
/// on the board and walkable, a trap only where no active unit stands, each once.
pub fn cells(w: &World, by: UnitId, def: &HazardDef, p: &Place, anchor: V2, aim: V2, area: &[V2]) -> Vec<V2> {
    let raw: Vec<V2> = match p.shape {
        Shape::Skill if !area.is_empty() => area.to_vec(),
        Shape::Skill | Shape::One => vec![anchor],
        Shape::Plus => vec![anchor, anchor + V2::new(1, 0), anchor + V2::new(-1, 0), anchor + V2::new(0, 1), anchor + V2::new(0, -1)],
        Shape::Burst => (-1..=1).flat_map(|dx| (-1..=1).map(move |dy| anchor + V2::new(dx, dy))).collect(),
        Shape::Ring => skill::ring_cells(if p.at_self { w.pos(by) } else { anchor }),
        Shape::Line => {
            let d = shape_heading(w, by, aim);
            let start = if p.at_self { w.pos(by) + d } else { anchor };
            (0..3).map(|i| start + d * i).collect()
        }
        Shape::Arc => {
            // FB-106: drawn as the skill's ARC from the user's tile; a source on the user's own tile reads the heading
            let (pos, d) = (w.pos(by), shape_heading(w, by, aim));
            let source = if p.at_self { pos + d } else { anchor };
            skill::arc_cells(if source == pos { source - d } else { pos }, source)
        }
    };
    let mut out: Vec<V2> = Vec::new();
    for c in raw {
        if w.grid.is_walkable(c) && !out.contains(&c) && !(def.trap && w.is_occupied(c)) {
            out.push(c);
        }
    }
    out
}

/// GH2-GH4, GH7: leaves `p` from `by` in round `round`; the newest hazard on a tile replaces the old. None when the row
/// is unknown or no tile takes it.
#[allow(clippy::too_many_arguments)]
fn place(w: &mut World, round: i32, by: UnitId, p: &Place, anchor: V2, aim: V2, area: &[V2], skill: &str) -> Option<Placement> {
    let def = w.cat.hazards.get(&p.id)?.clone();
    let cells = cells(w, by, &def, p, anchor, aim, area);
    if cells.is_empty() {
        return None;
    }
    let rounds = if p.rounds > 0 { p.rounds } else { def.duration.max(1) };
    for c in &cells {
        w.grid.ground.shift_remove(c);
        w.grid.ground.insert(*c, Placed { def: def.clone(), by, round, last_round: round + rounds - 1 });
    }
    let at = if cells.len() == 1 { cells[0].to_string() } else { format!("{} tiles", cells.len()) };
    let line = format!("[Hazard] {} leaves {} on {at} for {rounds} round{}", w.units[by].name, def.name, if rounds == 1 { "" } else { "s" });
    Some(Placement { by, hazard: def.name.clone(), id: def.id.clone(), skill: skill.into(), cells, rounds, line })
}

fn record(res: &mut Resolver, w: &mut World, p: Option<Placement>) {
    if let Some(p) = p {
        res.log.push(p.line.clone());
        w.ground.events.push(Event::Placed(p));
    }
}

/// GH2 (step 7 of a landed hit, after the forced moves): the placement of the Attack, Area, Support or item action `idx`
/// that just landed: at the aim tile or the target unit's tile, or the user's. Nothing when it failed or the item was
/// not used.
pub fn after_impact(res: &mut Resolver, w: &mut World, idx: usize) {
    if !on(w) {
        return;
    }
    let a = &res.scheduled[idx];
    let Some(p) = a.command.hazard.clone() else { return };
    let by = a.unit;
    if a.command.failed || !w.has(by) {
        return;
    }
    let unit_tile = |t: UnitId| w.grid.pos(t);
    let (target, area): (Option<V2>, Vec<V2>) = match &a.command.kind {
        CmdKind::Area { cells, aim, .. } => (Some(*aim), cells.clone()),
        CmdKind::Attack { target, last_target, .. } => (last_target.or(*target).or(w.units[by].active_target).and_then(unit_tile), Vec::new()),
        CmdKind::Support { target, .. } => (unit_tile(*target), Vec::new()),
        CmdKind::Item { target, used: true, .. } => (unit_tile(*target), Vec::new()),
        _ => return,
    };
    let me = w.pos(by);
    let anchor = if p.at_self { me } else if let Some(t) = target { t } else { return };
    let skill = a.command.name.clone();
    let placed = place(w, res.current_round, by, &p, anchor, target.unwrap_or(me), &area, &skill);
    record(res, w, placed);
}

/// GH2: a Defend's placement at its start (with its grants), always on the user's tile.
pub fn on_start(res: &mut Resolver, w: &mut World, idx: usize) {
    if !on(w) {
        return;
    }
    let a = &res.scheduled[idx];
    let Some(p) = a.command.hazard.clone() else { return };
    let by = a.unit;
    if !a.command.is_defend() || !w.has(by) {
        return;
    }
    let me = w.pos(by);
    let skill = a.command.name.clone();
    let placed = place(w, res.current_round, by, &Place { at_self: true, ..p }, me, me, &[], &skill);
    record(res, w, placed);
}

// ---- Acting on units (GH5, GH6) --------------------------------------------------------------------------------------

/// GH6: the name of the trap on `cell` that `u`, standing there, springs (one that affects it); the move stops there.
pub fn springs(w: &World, u: UnitId, cell: V2) -> Option<String> {
    let p = w.grid.ground.get(&cell)?;
    (w.grid.pos(u) == Some(cell) && trap_at(w, u, cell)).then(|| p.def.name.clone())
}

/// GH6: a trap on `cell` that `u` would spring by entering it (rule on; the forced-move preview reads it before the move).
pub fn trap_at(w: &World, u: UnitId, cell: V2) -> bool {
    w.grid.ground.get(&cell).is_some_and(|p| p.def.trap && on(w) && p.affects(w, u))
}

/// GH5, GH6: the hazard on `cell` meets `u` as it enters (after the terrain's roll; the first tile of each hazard in a
/// move acts, `seen` holds the ids met). The note for the move's line; the hit is recorded for the session.
pub fn enter(w: &mut World, rng: &mut GodotRng, u: UnitId, cell: V2, seen: &mut Vec<String>) -> Option<String> {
    let p = w.grid.ground.get(&cell)?;
    if !on(w) || !p.acts_on_enter() || !p.affects(w, u) || seen.contains(&p.def.id) || !w.units[u].is_active() {
        return None;
    }
    seen.push(p.def.id.clone());
    let hit = apply(w, rng, u, cell, false)?;
    let line = hit.line.clone();
    if hit.stop {
        w.ground.stops.push(w.ground.events.len());
    }
    w.ground.events.push(Event::Hit(hit));
    Some(line)
}

/// A4.2 (C08): a ground hazard row's status as an EffectSpec: an inflict (rolled when its row is `resistable`) carrying
/// the row's `potency`, `status_rounds` and `magnitude`, with `source` (the placer's name).
pub fn effect_of(def: &HazardDef, source: &str) -> status::EffectSpec {
    status::EffectSpec::inflict(&def.status, def.potency, def.status_rounds, def.magnitude, source)
}

/// GH5, GH6: the hazard on `cell` acts on `u`: its flat damage through guard (not armour), then its status roll (one
/// draw, the placer as the source); a trap entered is removed. None when nothing happened (a resisted roll alone).
fn apply(w: &mut World, rng: &mut GodotRng, u: UnitId, cell: V2, round_end: bool) -> Option<Hit> {
    let p = w.grid.ground.get(&cell)?.clone();
    let mark = w.hp_journal.as_ref().map_or(0, |j| j.len());
    let before = w.units[u].hp;
    if p.def.damage > 0 {
        w.damage(u, p.def.damage, false);
    }
    // On enter the rows wait for the hazard's own cell; at round end the metrics read them as round-end damage
    let journal = if round_end { Vec::new() } else { w.hp_journal.as_mut().map(|j| j.split_off(mark.min(j.len()))).unwrap_or_default() };
    let lost = before - w.units[u].hp;
    let mut landed = String::new();
    if !p.def.status.is_empty() && w.units[u].is_alive() {
        // A4.2: the hazard row as an EffectSpec (the placer as the source)
        let spec = effect_of(&p.def, &w.units[p.by].name);
        if status::apply_effect(&mut w.units[u], &spec, status::Timing::Next, 0.0, Some(rng)) {
            landed = p.def.status.clone();
        }
    }
    let trap = p.def.trap && !round_end;
    if trap {
        w.grid.ground.shift_remove(&cell);
    }
    let stop = p.def.stop_moves && !round_end;
    if lost == 0 && landed.is_empty() && !trap && !stop {
        return None;
    }
    let name = &w.units[u].name;
    let mut line = if trap {
        format!("[Hazard] {name} springs the {}", p.def.name)
    } else if round_end {
        format!("[Hazard] {name} is caught in {}", p.def.name)
    } else {
        format!("[Hazard] {name} enters {}", p.def.name)
    };
    let mut parts = Vec::new();
    if lost > 0 {
        parts.push(format!("{lost} damage"));
    }
    if !landed.is_empty() {
        parts.push(status::display_name(&landed));
    }
    if !w.units[u].is_alive() {
        parts.push("defeated".into());
    }
    if !parts.is_empty() {
        line += &format!(": {}", parts.join(", "));
    }
    Some(Hit { unit: u, by: p.by, hazard: p.def.name.clone(), id: p.def.id.clone(), damage: lost, status: landed, trap, round_end, stop, stopped: 0, line, journal })
}

/// GH6 `stop_moves`: cancels the Walks of this round that have not landed of each unit a hazard stopped since the last
/// call (the resolver calls it after an action starts or lands). Any other path that moves a unit onto hazards (a
/// `forced::shove_unit` from a trigger or an arena push) must call it too, then the session's `log_ground`.
pub fn settle(res: &mut Resolver, w: &mut World) {
    for i in std::mem::take(&mut w.ground.stops) {
        let Some(Event::Hit(h)) = w.ground.events.get_mut(i) else { continue };
        h.stopped = crate::forced::stop_walks(res, h.unit);
        if h.stopped > 0 {
            h.line += "; its moves this round are cancelled";
        }
    }
}

/// GH5, GH7 (after the terrain's round-end rolls): each active unit on a round-end hazard it is affected by, in
/// registration order, takes it (its damage is a round-end step credited to the placer); then the hazards whose last
/// round this is fade.
pub fn round_end(res: &mut Resolver, w: &mut World) {
    if w.grid.ground.is_empty() {
        return;
    }
    if on(w) {
        for u in res.registered.clone() {
            if !w.has(u) {
                continue;
            }
            let cell = w.pos(u);
            let Some(p) = w.grid.ground.get(&cell) else { continue };
            if !p.acts_at_round_end() || !p.affects(w, u) {
                continue;
            }
            let Some(h) = apply(w, &mut res.rng, u, cell, true) else { continue };
            if h.damage > 0 {
                res.round_end_steps.push(RoundEndStep { unit: u, status: "hazard", amount: h.damage, source: w.units[h.by].name.clone() });
            }
            res.log.push(h.line.clone());
            w.ground.events.push(Event::Hit(h));
        }
    }
    let round = res.current_round;
    let gone: Vec<V2> = w.grid.ground.iter().filter(|(_, p)| p.last_round <= round).map(|(c, _)| *c).collect();
    let mut fades: Vec<Fade> = Vec::new();
    for c in gone {
        let Some(p) = w.grid.ground.shift_remove(&c) else { continue };
        match fades.iter_mut().find(|f| f.id == p.def.id && f.by == p.by) {
            Some(f) => f.cells += 1,
            None => fades.push(Fade { by: p.by, hazard: p.def.name.clone(), id: p.def.id.clone(), cells: 1, line: format!("[Hazard] {} fades", p.def.name) }),
        }
    }
    for f in fades {
        res.log.push(f.line.clone());
        w.ground.events.push(Event::Faded(f));
    }
}

// ---- What a side sees (GH6, GH8) -----------------------------------------------------------------------------------

/// GH6: the hazard on `cell` as `viewer`'s side sees it (None: none, or hidden from that side).
pub fn seen_at(w: &World, viewer: UnitId, cell: V2) -> Option<&Placed> {
    w.grid.ground.get(&cell).filter(|p| p.seen_by(w, viewer))
}

/// GH8: a hazard on `cell` that would harm `c` and that its side can see (rule on).
pub fn harms(w: &World, c: UnitId, cell: V2) -> bool {
    w.grid.ground.get(&cell).is_some_and(|p| p.harmful() && p.affects(w, c) && p.seen_by(w, c)) && on(w)
}

/// GH8: the AI's extra cost of entering `cell` for `c`: AI_HAZARD_PENALTY for a terrain hazard or a ground hazard that
/// `harms` it.
pub fn ai_penalty(w: &World, c: UnitId, cell: V2) -> i32 {
    if w.grid.has_hazard(cell) || harms(w, c, cell) {
        AI_HAZARD_PENALTY
    } else {
        0
    }
}
