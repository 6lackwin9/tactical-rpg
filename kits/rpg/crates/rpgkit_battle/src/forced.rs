//! Forced movement (HOOKS1 5, rule `forced_move`; engine-only, no Godot original). An attack skill's `forced_move`
//! pushes or pulls each unit it damaged that is still on the field, after the hit's stagger and breaks (step 6 of a
//! landed hit, HOOKS1 4.3; FM1): one tile at a time on the dominant axis of `target - origin`, a tie on the axis of the
//! attacker's facing, else x (FM2), until a step is blocked by the arena edge, a tile that is not walkable, an exit or an
//! active unit (FM3). A blocked step with tiles left is a collision: the moved unit takes `collide_mult` x the
//! attacker's power through its own armour and guard (no crit roll) and `collide_stagger` through P6, a `collide_break`
//! breaks it as P14 does; a unit that blocked it takes the same hit and stagger (FM4). Passive and item `forced_move`
//! effects hold a unit firm or add tiles to its own pushes (FM5). Each tile entered rolls its hazards as a walk does,
//! and a ground trap the unit springs stops the move on its tile (FM6, HOOKS1 GH6); the move is not the unit's own move and costs nothing, and `stop_moves` cancels its Walks that have not landed
//! (FM7). `shove` is the step rule the trigger effect `move` and an arena `push` reuse with a direction and a flat
//! damage (FM9). Nothing here draws a random number but the hazards' existing status rolls (no crit, no break chance),
//! and every entry point reads the rule first, so classic battles never move, log or record anything new.

use crate::command::{hazard_on_enter, terrain_name, CmdKind};
use crate::grid::{V2, ZERO};
use crate::poise::{self, BreakCause, StaggerHit};
use crate::resolver::{Resolver, ScheduledAction};
use crate::round_intent;
use crate::skill::{self, Kind};
use crate::world::{UnitId, World};
use rpgkit_data::SkillDef;
use rpgkit_rules::damage::{self, HitInput};
use rpgkit_rules::ruleset;

/// FM5: the most tiles a forced move covers with its holder's `bonus_tiles`.
pub const MAX_TILES: i32 = 4;

/// Whether the battle plays by forced movement.
pub fn on(w: &World) -> bool {
    w.ruleset.has(ruleset::FORCED_MOVE)
}

/// FM1: when a skill's forced move acts on a unit it damaged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum When {
    Always,
    /// A move entry of the target (a Walk that moved, a Disengage that landed) landed this round.
    Moved,
    /// The target was in windup (P5) as the hit landed.
    Windup,
}

/// FM1: an attack skill's `forced_move`, carried by its command (read only under the rule).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shove {
    pub pull: bool,
    /// 1-3 (the attacker's `bonus_tiles` are added when it lands, FM5).
    pub tiles: i32,
    /// From the area's aim cell instead of the attacker (an area skill; a target on the aim cell uses the attacker).
    pub from_center: bool,
    pub when: When,
    pub stop_moves: bool,
    pub collide_mult: f64,
    pub collide_stagger: i32,
    pub collide_break: bool,
}

impl Shove {
    /// The skill's forced move: ATTACK skills only (items never carry one); None without the field.
    pub fn of(s: &SkillDef) -> Option<Self> {
        if skill::is_item(s) || skill::kind(s) != Kind::Attack {
            return None;
        }
        let f = s.forced_move.as_ref()?;
        let when = match f.when.as_str() {
            "moved" => When::Moved,
            "windup" => When::Windup,
            _ => When::Always,
        };
        Some(Self {
            pull: f.kind == "pull",
            tiles: f.tiles.clamp(1, 3),
            from_center: f.from == "center",
            when,
            stop_moves: f.stop_moves,
            collide_mult: f.collide_mult.clamp(0.0, 3.0),
            collide_stagger: f.collide_stagger.clamp(0, 50),
            collide_break: f.collide_break,
        })
    }
}

/// FM4, FM9: what a collision deals to the moved unit and to a unit that blocked it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Collide {
    /// The unit whose power drives the hit (`mult` x its power through armour and guard, no crit); None = `flat`.
    pub attacker: Option<UnitId>,
    pub mult: f64,
    pub ranged: bool,
    pub offhand: bool,
    /// Flat damage through guard, not armour (a trigger `move`, an arena `push`; read when `attacker` is None).
    pub flat: i32,
    /// Stagger through P6 (rule poise).
    pub stagger: i32,
    /// The moved unit breaks as P14 does when in windup (not when unbreakable or steadfast); never the blocker.
    pub sure_break: bool,
    /// The length of the action P6's passive `stagger_taken` factor reads (1 for a trigger or a hazard).
    pub blocks: i32,
}

/// One collision hit: the unit, the HP it lost, the log line, its stagger record (rule poise), and its HP journal rows
/// held back so the battle metrics hear them with the collision's own log cell.
#[derive(Debug, Clone, PartialEq)]
pub struct CollisionHit {
    pub unit: UnitId,
    pub damage: i32,
    pub line: String,
    pub stagger: Option<StaggerHit>,
    pub journal: Vec<(UnitId, i32)>,
}

/// One forced move: who drove whom, from where to where, what stopped it, its collision hits (the moved unit's first,
/// then the blocker's), the Walks it cancelled and the log line (hazard notes included).
#[derive(Debug, Clone, PartialEq)]
pub struct Moved {
    pub unit: UnitId,
    /// The unit that drove it (None: an arena hazard).
    pub by: Option<UnitId>,
    pub pull: bool,
    pub from: V2,
    pub to: V2,
    pub tiles: i32,
    /// FM5: it held firm (no move, no collision).
    pub held: bool,
    /// What blocked a step with tiles left ("" = no collision): a unit's name, "the arena edge", "the exit", "the
    /// <terrain>".
    pub blocked_by: String,
    /// The unit that blocked it.
    pub blocker: Option<UnitId>,
    pub hits: Vec<CollisionHit>,
    /// FM7: the Walks of the moved unit `stop_moves` cancelled.
    pub stopped: i32,
    pub line: String,
}

/// FM1: the units in windup at the impact of the action `idx`, read before its stagger can break them (empty unless the
/// rule is on and the action's forced move asks for a windup).
pub fn windups(res: &Resolver, w: &World, idx: usize) -> Vec<UnitId> {
    let a = &res.scheduled[idx];
    if !on(w) || a.command.forced.is_none_or(|f| f.when != When::Windup) {
        return Vec::new();
    }
    a.command.hits.iter().map(|(t, _)| *t).filter(|t| poise::in_windup(&res.scheduled, *t, a.impact)).collect()
}

/// FM1-FM8: the forced moves of the action `idx` that just landed (its stagger and breaks done): each unit it damaged
/// that is still on the field, in hit order, once. The records go to the command's `pushes`, the lines to the
/// resolver's log. `windups`: the units in windup as the hit landed (`windups`).
pub fn resolve(res: &mut Resolver, w: &mut World, idx: usize, windups: &[UnitId]) {
    if !on(w) {
        return;
    }
    let Some(shove) = res.scheduled[idx].command.forced else { return };
    let src = res.scheduled[idx].unit;
    let Some(src_pos) = w.grid.pos(src) else { return };
    let block = res.scheduled[idx].impact;
    let ranged_unit = w.units[src].attack_range > 1;
    let (aim, ranged, offhand) = match &res.scheduled[idx].command.kind {
        CmdKind::Area { aim, .. } => (Some(*aim), ranged_unit, false),
        CmdKind::Attack { use_offhand, .. } => (None, ranged_unit && !*use_offhand, *use_offhand),
        _ => return,
    };
    let tiles = (shove.tiles + w.units[src].forced_traits().bonus_tiles).min(MAX_TILES);
    let collide = Collide { attacker: Some(src), mult: shove.collide_mult, ranged, offhand, flat: 0, stagger: shove.collide_stagger, sure_break: shove.collide_break, blocks: res.scheduled[idx].command.block_cost };
    let mut targets: Vec<UnitId> = Vec::new();
    for (t, _) in &res.scheduled[idx].command.hits {
        if !targets.contains(t) {
            targets.push(*t);
        }
    }
    let mut out = Vec::new();
    for t in targets {
        if t == src || !w.has(t) {
            continue;
        }
        let applies = match shove.when {
            When::Always => true,
            When::Moved => moved_this_round(&res.scheduled, t),
            When::Windup => windups.contains(&t),
        };
        if !applies {
            continue;
        }
        let at = w.pos(t);
        let origin = match aim {
            Some(a) if shove.from_center && at != a => a,
            _ => src_pos,
        };
        let dir = direction(w, src, origin, at, shove.pull);
        let toward = shove.pull.then_some(origin);
        out.push(shove_unit(res, w, t, Some(src), dir, tiles, toward, shove.stop_moves, &collide, block));
    }
    res.scheduled[idx].command.pushes = out;
}

/// FM1 `when` moved: a move entry of `t` landed this round (a Walk that moved at least a tile, a Disengage that
/// landed). A forced move or a carried move is not one (FM7).
pub fn moved_this_round(sched: &[ScheduledAction], t: UnitId) -> bool {
    sched.iter().any(|a| {
        a.unit == t
            && a.landed
            && match &a.command.kind {
                CmdKind::Walk { moved_tiles, .. } => *moved_tiles > 0,
                CmdKind::Disengage { landed, .. } => *landed,
                _ => false,
            }
    })
}

/// FM2: the one-tile step of a push (away from `origin`) or a pull (toward it) of the unit at `at`: the dominant axis
/// of `at - origin`; a diagonal tie takes the axis of `by`'s facing (RULESET1 MV1) when that facing is straight, else x.
/// ZERO when `at` is the origin.
pub fn direction(w: &World, by: UnitId, origin: V2, at: V2, pull: bool) -> V2 {
    let d = at - origin;
    if d == ZERO {
        return ZERO;
    }
    let facing = round_intent::facing(w, by);
    let along_y = d.y.abs() > d.x.abs() || (d.y.abs() == d.x.abs() && facing.x == 0 && facing.y != 0);
    let step = if along_y { V2::new(0, d.y.signum()) } else { V2::new(d.x.signum(), 0) };
    if pull {
        step * -1
    } else {
        step
    }
}

/// FM3-FM7, FM9: moves `t` up to `tiles` tiles by `dir` at `block` (a pull passes its origin as `toward`: it stops at
/// distance 1, or once level with the origin on its axis). A unit whose `forced_move` effects make it immune holds firm
/// (FM5). The entered tiles roll their hazards (FM6); `stop` cancels its Walks that have not landed (FM7); a step
/// refused with tiles left is a collision (FM4). Logs one line for the move and one per collision hit.
///
/// Callers outside a landed hit (the trigger effect `move`, HOOKS1-H; an arena `push`, HOOKS1-E) must afterwards call
/// `ground::settle` (a trap's `stop_moves` cancels the moved unit's later Walks) and, in a session, `log_ground` (the
/// hazard hits of the entered tiles get their log cells and metrics); the resolver does both for hits.
#[allow(clippy::too_many_arguments)]
pub fn shove_unit(res: &mut Resolver, w: &mut World, t: UnitId, by: Option<UnitId>, dir: V2, tiles: i32, toward: Option<V2>, stop: bool, collide: &Collide, block: i32) -> Moved {
    let from = w.grid.pos(t).unwrap_or(crate::grid::NO_CELL);
    let mut m = Moved { unit: t, by, pull: toward.is_some(), from, to: from, tiles: 0, held: false, blocked_by: String::new(), blocker: None, hits: Vec::new(), stopped: 0, line: String::new() };
    let traits = w.units[t].forced_traits();
    if traits.immune_always || (traits.immune_guarding && crate::trigger::guarding(w, &res.scheduled, t, block)) {
        m.held = true;
        m.line = format!("[Hold] {} holds firm", w.units[t].name);
        res.log.push(m.line.clone());
        return m;
    }
    let mut entered = Vec::new();
    let mut refused = None;
    if dir != ZERO {
        for _ in 0..tiles {
            let here = w.pos(t);
            if let Some(o) = toward {
                let gap = o - here;
                if here.dist(o) <= 1 || gap.x * dir.x + gap.y * dir.y <= 0 {
                    break;
                }
            }
            let next = here + dir;
            // FM3: the edge, a tile that is not walkable, an exit (a forced move never withdraws) or an active unit
            if !w.jump_unit(t, next) {
                refused = Some(next);
                break;
            }
            entered.push(next);
            // FM6, HOOKS1 GH6: a trap the unit springs stops the move on its tile (no collision; rule ground_hazards)
            if crate::ground::springs(w, t, next).is_some() {
                break;
            }
        }
    }
    m.to = w.pos(t);
    m.tiles = entered.len() as i32;
    let notes = hazard_on_enter(w, &mut res.rng, t, &entered);
    if stop {
        m.stopped = stop_walks(res, t);
    }
    m.line = move_line(w, &m, &notes);
    res.log.push(m.line.clone());
    if let Some(cell) = refused {
        m.blocked_by = obstacle(w, cell);
        m.blocker = w.occupant(cell);
        let hit = collision_hit(res, w, t, collide, block, true, &format!("hits {}", m.blocked_by));
        m.hits.push(hit);
        if let Some(b) = m.blocker {
            let hit = collision_hit(res, w, b, collide, block, false, &format!("is struck by {}", w.units[t].name));
            m.hits.push(hit);
        }
    }
    m
}

/// FM7: cancels `t`'s Walks of this round that have not landed; how many (also HOOKS1 GH6's `stop_moves`).
pub(crate) fn stop_walks(res: &mut Resolver, t: UnitId) -> i32 {
    let mut n = 0;
    for a in res.scheduled.iter_mut().filter(|a| a.unit == t && !a.cancelled && !a.landed && matches!(a.command.kind, CmdKind::Walk { .. })) {
        a.cancelled = true;
        n += 1;
    }
    n
}

/// FM9: `[Push] A drives B 2 tiles to (8, 4)` (a pull drags; no driver: "is driven"), the cancelled Walks,
/// then the hazard notes.
fn move_line(w: &World, m: &Moved, notes: &[String]) -> String {
    let name = &w.units[m.unit].name;
    let tiles = format!("{} tile{}", m.tiles, if m.tiles == 1 { "" } else { "s" });
    let tag = if m.pull { "Pull" } else { "Push" };
    let mut line = match m.by {
        Some(b) if m.pull => format!("[{tag}] {} drags {name} {tiles} to {}", w.units[b].name, m.to),
        Some(b) => format!("[{tag}] {} drives {name} {tiles} to {}", w.units[b].name, m.to),
        None => format!("[{tag}] {name} is driven {tiles} to {}", m.to),
    };
    if m.stopped > 0 {
        line += "; its moves this round are cancelled";
    }
    for n in notes {
        line += &format!(" {n}");
    }
    line
}

/// FM4: what refused a step into `cell`: the unit there, the arena edge, the exit, else the tile's terrain (its catalog
/// name, else its id: "the wall").
fn obstacle(w: &World, cell: V2) -> String {
    if let Some(o) = w.occupant(cell) {
        return w.units[o].name.clone();
    }
    if !w.grid.is_valid(cell) {
        return "the arena edge".into();
    }
    if w.grid.is_exit(cell) {
        return "the exit".into();
    }
    let id = w.grid.terrain_id(cell);
    let name = if w.cat.terrain.contains_key(id) { terrain_name(w, cell) } else { id.replace('_', " ") };
    format!("the {}", name.to_lowercase())
}

/// FM4: one collision hit on `v` (`moved`: the unit that was moved; else the unit that blocked it): the damage (HP lost
/// is recorded), its line `[Collision] <v> <what>: <n> damage`, then the stagger. The HP journal rows of the damage are
/// held back in the record (the session hands them to the metrics with the collision's cell).
fn collision_hit(res: &mut Resolver, w: &mut World, v: UnitId, c: &Collide, block: i32, moved: bool, what: &str) -> CollisionHit {
    let mark = w.hp_journal.as_ref().map_or(0, |j| j.len());
    let before = w.units[v].hp;
    if w.units[v].is_alive() {
        match c.attacker {
            Some(a) if c.mult > 0.0 => {
                let hit = damage::compute(&w.units[a], &w.units[v], HitInput { ranged: c.ranged, offhand: c.offhand, skill_mult: c.mult, ..HitInput::default() });
                w.damage(v, hit.final_dmg, true);
            }
            None if c.flat > 0 => {
                w.damage(v, c.flat, false);
            }
            _ => {}
        }
    }
    let journal = w.hp_journal.as_mut().map(|j| j.split_off(mark.min(j.len()))).unwrap_or_default();
    let lost = before - w.units[v].hp;
    let mut line = format!("[Collision] {} {what}", w.units[v].name);
    if lost > 0 {
        line += &format!(": {lost} damage");
    }
    if !w.units[v].is_alive() {
        line += ", defeated";
    }
    res.log.push(line.clone());
    let stagger = collision_stagger(res, w, v, c, block, moved);
    CollisionHit { unit: v, damage: lost, line, stagger, journal }
}

/// FM4 (rule poise): a collision's stagger on `v`: P6's factors (no flank), P7's break at 0 poise, P12's hold at 1, and
/// for the moved unit of a `collide_break` collision P14's break of a windup. No break chance is rolled (P13 draws; a
/// collision draws nothing). None without a bar or anything to apply. The trigger effect `stagger` reuses it (HOOKS1
/// 8.2: stagger through P6, `moved` false).
pub(crate) fn collision_stagger(res: &mut Resolver, w: &mut World, v: UnitId, c: &Collide, block: i32, moved: bool) -> Option<StaggerHit> {
    let sure = moved && c.sure_break;
    if !poise::on(w) || (c.stagger <= 0 && !sure) {
        return None;
    }
    let st = w.poise.get(&v).copied()?;
    if !w.units[v].is_alive() || st.max <= 0 {
        return None;
    }
    let windup = poise::in_windup(&res.scheduled, v, block);
    let f = poise::factors(w, &res.scheduled, v, block, c.blocks.max(1), false);
    let taken = poise::stagger_taken(c.stagger.max(0), &f);
    let mut hit = StaggerHit { target: v, stagger: c.stagger.max(0), factors: f, windup, taken, before: st.value, after: st.value, chance: None, broke: None };
    // P8: a unit broken this round stays at 0 and breaks no further
    if st.broken {
        return Some(hit);
    }
    let held = poise::unbreakable(w, &res.scheduled, v, block);
    let mut after = st.value - taken;
    if held {
        after = after.max(st.value.min(1));
    }
    let cause = if taken > 0 && after <= 0 && !held {
        Some(BreakCause::Poise)
    } else if windup && sure && !held {
        Some(BreakCause::Collision)
    } else {
        None
    };
    if let Some(bar) = w.poise.get_mut(&v) {
        bar.value = after.max(0);
    }
    hit.after = after.max(0);
    if let Some(cause) = cause {
        hit.after = 0;
        hit.broke = Some(poise::break_unit(res, w, v, block, cause));
    }
    Some(hit)
}
