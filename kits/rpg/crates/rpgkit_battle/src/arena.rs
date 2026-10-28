//! Scheduled arena hazards (HOOKS1 7, rule `arena_hazards`; engine-only, no Godot original). An encounter's
//! `arena_hazards` (0-4 rows, AH1) strike struck cells (`cells`, whole `columns` or whole `rows`) at the end of each
//! listed block of each listed round (`rounds` [] = every round), after the block's actions and before the round-end
//! effects, the phase checks and the verdict (AH2). Every active, placed, unreserved unit on a struck cell, either side,
//! takes the `effect` in registration order: damage (flat or `damage_pct` of max HP, through guard, not armour), the
//! status roll (the hazard's name as the source), stagger through P6 (a vent into a windup can break it; rule poise),
//! then a `push` by a compass direction with a flat `collide_damage` (FM9; rule forced_move), after which the ground
//! settles (a trap's `stop_moves`) as every caller of `forced::shove_unit` outside a landed hit must. Terrain never
//! changes. The struck cells and blocks of a round are known at its start (AH3): `due` lists them (the tells record,
//! `intent::arena_tells`) and PL4's threat
//! count adds 1 for a cell due this round (`due_on`, AT3). Nothing is read with the rule off: the schedule stays empty
//! in classic, so classic battles never strike, draw, log or record anything new.

use crate::forced::{self, Collide, Moved};
use crate::grid::{V2, GRID_H, GRID_W};
use crate::poise::StaggerHit;
use crate::resolver::Resolver;
use crate::world::{UnitId, World};
use aegis_core::value::{self, Obj};
use rpgkit_rules::{ruleset, status};
use serde_json::Value;

/// Whether the battle plays by arena hazards.
pub fn on(w: &World) -> bool {
    w.ruleset.has(ruleset::ARENA_HAZARDS)
}

/// AH2 `push`: a compass step, 1-3 tiles, flat collision damage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Push {
    pub dir: V2,
    pub tiles: i32,
    pub collide_damage: i32,
}

/// AH2: what a strike does to each unit on its cells (0 / "" / None = that part is off).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Effect {
    pub damage: i32,
    pub damage_pct: i32,
    pub stagger: i32,
    pub status: String,
    pub potency: i32,
    pub magnitude: i32,
    pub status_rounds: i32,
    pub push: Option<Push>,
}

/// AH1: one scheduled arena hazard of the encounter.
#[derive(Debug, Clone, PartialEq)]
pub struct Hazard {
    pub id: String,
    pub name: String,
    /// The struck cells on the board, each once (`columns` top to bottom, `rows` left to right).
    pub cells: Vec<V2>,
    /// The rounds it strikes in (empty = every round).
    pub rounds: Vec<i32>,
    /// The blocks at whose end it strikes.
    pub blocks: Vec<i32>,
    pub effect: Effect,
    /// The strike's event line ("" = "<name> strikes.").
    pub line: String,
}

/// AH2 `dir`: east, west, north (up the board) or south.
fn dir_of(s: &str) -> Option<V2> {
    match s {
        "east" => Some(V2::new(1, 0)),
        "west" => Some(V2::new(-1, 0)),
        "north" => Some(V2::new(0, -1)),
        "south" => Some(V2::new(0, 1)),
        _ => None,
    }
}

fn ints(o: &Obj, key: &str) -> Vec<i32> {
    value::array(o, key).iter().map(|v| value::to_int(v, -1)).collect()
}

impl Hazard {
    /// AH1: a row as stored (the content check validates it; out-of-board cells and unknown directions are dropped).
    pub fn from_obj(o: &Obj) -> Self {
        let on_board = |c: &V2| c.x >= 0 && c.x < GRID_W && c.y >= 0 && c.y < GRID_H;
        let mut cells: Vec<V2> = Vec::new();
        let mut add = |c: V2| {
            if on_board(&c) && !cells.contains(&c) {
                cells.push(c);
            }
        };
        for p in value::array(o, "cells") {
            if let Some(p) = p.as_array().filter(|p| p.len() == 2) {
                add(V2::new(value::to_int(&p[0], -1), value::to_int(&p[1], -1)));
            }
        }
        for x in ints(o, "columns") {
            (0..GRID_H).for_each(|y| add(V2::new(x, y)));
        }
        for y in ints(o, "rows") {
            (0..GRID_W).for_each(|x| add(V2::new(x, y)));
        }
        let e = value::obj(o, "effect").cloned().unwrap_or_default();
        let push = value::obj(&e, "push").and_then(|p| {
            let dir = dir_of(&value::string(p, "dir", ""))?;
            Some(Push { dir, tiles: value::int(p, "tiles", 1).clamp(1, 3), collide_damage: value::int(p, "collide_damage", 0).clamp(0, 50) })
        });
        let effect = Effect {
            damage: value::int(&e, "damage", 0).clamp(0, 99),
            damage_pct: value::int(&e, "damage_pct", 0).clamp(0, 50),
            stagger: value::int(&e, "stagger", 0).clamp(0, 50),
            status: value::string(&e, "status", ""),
            potency: value::int(&e, "potency", 100),
            magnitude: value::int(&e, "magnitude", 0),
            status_rounds: value::int(&e, "status_rounds", 1),
            push,
        };
        Self { id: value::string(o, "id", ""), name: value::string(o, "name", ""), cells, rounds: ints(o, "rounds"), blocks: ints(o, "blocks"), effect, line: value::string(o, "line", "") }
    }

    /// AH1: it strikes in `round` (`rounds` [] = every round).
    pub fn due_in(&self, round: i32) -> bool {
        self.rounds.is_empty() || self.rounds.contains(&round)
    }

    /// AH2: it strikes at the end of `block` of `round`.
    pub fn strikes(&self, round: i32, block: i32) -> bool {
        self.due_in(round) && self.blocks.contains(&block)
    }
}

/// The battle's arena schedule: the encounter's hazards (empty with the rule off) and the round they are told for.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Arena {
    pub hazards: Vec<Hazard>,
    pub round: i32,
}

impl Arena {
    /// AH1: the encounter row's `arena_hazards` under the rule (none with it off, so classic stays as it was).
    pub fn of(raw: &Obj, rules: &ruleset::Ruleset) -> Self {
        if !rules.has(ruleset::ARENA_HAZARDS) {
            return Self::default();
        }
        let hazards = value::array(raw, "arena_hazards").iter().filter_map(Value::as_object).map(Hazard::from_obj).collect();
        Self { hazards, round: 0 }
    }
}

/// AH3: the round the tells and PL4 read (the session calls it as each round begins, before the AI plans).
pub fn round_start(w: &mut World, round: i32) {
    w.arena.round = round;
}

/// AH3: the hazards due in the told round, in list order (empty with the rule off; `intent::arena_tells` reads them).
pub fn due(w: &World) -> Vec<&Hazard> {
    if !on(w) {
        return Vec::new();
    }
    w.arena.hazards.iter().filter(|h| h.due_in(w.arena.round)).collect()
}

/// AT3: a hazard due this round strikes `cell` (PL4 counts it as a threat).
pub fn due_on(w: &World, cell: V2) -> bool {
    on(w) && w.arena.hazards.iter().any(|h| h.due_in(w.arena.round) && h.cells.contains(&cell))
}

/// What one struck unit took: the HP it lost, the status that landed ("" = none), its stagger record (rule poise), its
/// forced move (rule forced_move), the log line, and the HP journal rows of the damage held back so the metrics hear
/// them with the unit's own cell.
#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    pub unit: UnitId,
    pub damage: i32,
    pub status: String,
    pub stagger: Option<StaggerHit>,
    pub push: Option<Moved>,
    pub line: String,
    pub journal: Vec<(UnitId, i32)>,
}

/// One strike: the hazard (id, name), when, its cells, its event line and each unit it struck in registration order.
#[derive(Debug, Clone, PartialEq)]
pub struct Strike {
    pub id: String,
    pub name: String,
    pub round: i32,
    pub block: i32,
    pub cells: Vec<V2>,
    pub line: String,
    pub hits: Vec<Hit>,
}

/// AH2: the hazards that strike at the end of `block` of `round`, in list order; each takes the units standing on its
/// cells as it strikes (active, placed, not `reserved`) in registration order. Lines go to the resolver's log.
pub fn block_end(res: &mut Resolver, w: &mut World, round: i32, block: i32, reserved: &[UnitId]) -> Vec<Strike> {
    if !on(w) || w.arena.hazards.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for hz in w.arena.hazards.clone() {
        if !hz.strikes(round, block) {
            continue;
        }
        let victims: Vec<UnitId> = res.registered.iter().copied().filter(|&u| w.has(u) && !reserved.contains(&u) && hz.cells.contains(&w.pos(u))).collect();
        let line = if hz.line.is_empty() { format!("{} strikes.", hz.name) } else { hz.line.clone() };
        res.log.push(format!("[Arena] {line}"));
        let mut hits = Vec::new();
        for u in victims {
            if w.has(u) {
                hits.push(strike_unit(res, w, &hz, u, block));
            }
        }
        out.push(Strike { id: hz.id.clone(), name: hz.name.clone(), round, block, cells: hz.cells.clone(), line, hits });
    }
    out
}

/// AH2: `hz` strikes `u`: damage through guard (not armour), the status roll (one draw), stagger through P6, then the
/// push (its tiles roll their ground hazards; the ground settles after it).
fn strike_unit(res: &mut Resolver, w: &mut World, hz: &Hazard, u: UnitId, block: i32) -> Hit {
    let e = &hz.effect;
    let mark = w.hp_journal.as_ref().map_or(0, |j| j.len());
    let before = w.units[u].hp;
    let amount = if e.damage > 0 {
        e.damage
    } else if e.damage_pct > 0 {
        ((w.units[u].max_hp * e.damage_pct) as f64 / 100.0).round().max(1.0) as i32
    } else {
        0
    };
    if amount > 0 {
        w.damage(u, amount, false);
    }
    let journal = w.hp_journal.as_mut().map(|j| j.split_off(mark.min(j.len()))).unwrap_or_default();
    let lost = before - w.units[u].hp;
    let mut landed = String::new();
    if !e.status.is_empty() && w.units[u].is_alive() && status::try_inflict(&mut w.units[u], &e.status, e.potency, Some(&mut res.rng), e.status_rounds, e.magnitude, &hz.name) {
        landed = e.status.clone();
    }
    let stagger = if e.stagger > 0 && w.units[u].is_alive() {
        let c = Collide { attacker: None, mult: 0.0, ranged: false, offhand: false, flat: 0, stagger: e.stagger, sure_break: false, blocks: 1 };
        forced::collision_stagger(res, w, u, &c, block, false)
    } else {
        None
    };
    let name = w.units[u].name.clone();
    let mut parts = Vec::new();
    if lost > 0 {
        parts.push(format!("{lost} damage"));
    }
    if !landed.is_empty() {
        parts.push(status::display_name(&landed));
    }
    if let Some(st) = stagger.as_ref().filter(|st| st.taken > 0) {
        parts.push(format!("stagger {}", st.taken));
    }
    if !w.units[u].is_alive() {
        parts.push("defeated".into());
    }
    let mut line = format!("[Arena] {name} is caught by {}", hz.name);
    if !parts.is_empty() {
        line += &format!(": {}", parts.join(", "));
    }
    res.log.push(line.clone());
    let push = match e.push {
        Some(p) if forced::on(w) && w.has(u) => {
            let c = Collide { attacker: None, mult: 0.0, ranged: false, offhand: false, flat: p.collide_damage, stagger: 0, sure_break: false, blocks: 1 };
            let m = forced::shove_unit(res, w, u, None, p.dir, p.tiles, None, false, &c, block);
            crate::ground::settle(res, w);
            Some(m)
        }
        _ => None,
    };
    Hit { unit: u, damage: lost, status: landed, stagger, push, line, journal }
}
