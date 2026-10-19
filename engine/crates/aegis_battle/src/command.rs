//! BattleCommand and the commands (scripts/core/combat/battle_command.gd, scripts/combat/actions/*), plus the rules
//! they call: RiposteRules, SupportRules, HazardRules, CoverRules. RNG draws happen in Godot's order: hit crit,
//! status roll, riposte crit, per unit hit.

use crate::grid::{Grid, V2, NO_CELL};
use crate::path::{self, Cells, Hooks};
use crate::skill::{self, SkillId};
use crate::world::{UnitId, World};
use aegis_data::ItemDef;
use aegis_rules::damage::{self, Hit, HitInput};
use aegis_rules::rng::GodotRng;
use aegis_rules::stats::roundi;
use aegis_rules::{status, Stance, Unit};
use serde_json::Value;

pub const COUNTER_MULT: f64 = 0.5;

#[derive(Debug, Clone)]
pub struct Reaction {
    pub defender: UnitId,
    pub attacker: UnitId,
    pub damage: i32,
    pub crit: bool,
    pub defeated: bool,
}

#[derive(Debug, Clone)]
pub enum CmdKind {
    Attack {
        target: Option<UnitId>,
        skill_mult: f64,
        use_offhand: bool,
        follow_tiles: i32,
        dash_tiles: i32,
        status_rounds: i32,
        min_range: i32,
        max_range: i32,
        last_damage: i32,
        last_crit: bool,
        last_target: Option<UnitId>,
    },
    Area {
        cells: Vec<V2>,
        aim: V2,
        skill_mult: f64,
        status_rounds: i32,
        last_damage: i32,
        last_hits: i32,
        crit_hits: Vec<(UnitId, bool)>,
    },
    Defend {
        mitigation: f64,
        grant_status: String,
        status_rounds: i32,
        extra_grants: Vec<Value>,
        guard_broken: bool,
    },
    Focus {
        boost: f64,
        rounds: i32,
    },
    Rest {
        recover_ap: i32,
    },
    StatusBlock {
        status: String,
    },
    Support {
        target: UnitId,
        skill: SkillId,
        last_heal: i32,
    },
    Target {
        target: Option<UnitId>,
    },
    /// ItemCommand (ITEMUSE1 rules 3-5): one battle-pouch consumable on `target`, in the SupportCommand slot.
    Item {
        target: UnitId,
        item: Box<ItemDef>,
        used: bool,
        last_heal: i32,
        last_damage: i32,
        last_boost: f64,
    },
    Walk {
        path: Vec<V2>,
        expected_start: V2,
        goal: V2,
        moved_tiles: i32,
        cut_short: bool,
    },
    Disengage {
        dest: V2,
        max_tiles: i32,
        landed: bool,
    },
}

#[derive(Debug, Clone)]
pub struct Command {
    pub source: UnitId,
    pub priority: i32,
    pub block_cost: i32,
    pub name: String,
    pub ap_cost: i32,
    pub stance_switch_cost: i32,
    pub stance_on_start: Option<Stance>,
    pub stance_on_end: Option<Stance>,
    pub est_damage: i32,
    pub status_effect: String,
    pub status_potency: i32,
    pub status_magnitude: i32,
    pub failed: bool,
    pub disrupt_break: bool,
    pub exploit: Vec<Value>,
    pub exploit_consume: bool,
    pub broken_targets: Vec<UnitId>,
    pub exploited: Vec<(UnitId, String)>,
    pub reactions: Vec<Reaction>,
    pub log: String,
    pub kind: CmdKind,
}

impl Command {
    pub fn new(source: UnitId, priority: i32, block_cost: i32, kind: CmdKind) -> Self {
        Self {
            source,
            priority,
            block_cost,
            name: String::new(),
            ap_cost: 0,
            stance_switch_cost: 0,
            stance_on_start: None,
            stance_on_end: None,
            est_damage: 0,
            status_effect: String::new(),
            status_potency: 0,
            status_magnitude: 0,
            failed: false,
            disrupt_break: false,
            exploit: Vec::new(),
            exploit_consume: false,
            broken_targets: Vec::new(),
            exploited: Vec::new(),
            reactions: Vec::new(),
            log: String::new(),
            kind,
        }
    }

    pub fn initiative(&self, w: &World) -> i32 {
        self.priority * 1000 + w.units[self.source].speed
    }

    pub fn is_defend(&self) -> bool {
        matches!(self.kind, CmdKind::Defend { .. })
    }

    /// Runs when the command STARTS (after AP and stance): attack dash / chase, defend grants.
    pub fn on_start(&mut self, w: &mut World, rng: &mut GodotRng) -> String {
        let src = self.source;
        match &mut self.kind {
            CmdKind::Attack { target, follow_tiles, dash_tiles, min_range, max_range, .. } => {
                let tgt = effective_target(w, src, *target);
                let band = reach_band(w, src, *min_range, *max_range);
                if *dash_tiles > 0 {
                    return dash(w, rng, src, tgt, band, *dash_tiles);
                }
                let Some(t) = tgt else { return String::new() };
                if *follow_tiles <= 0 || !w.units[t].is_active() || !w.units[src].is_active() || in_reach(w, src, t, band) {
                    return String::new();
                }
                let from = w.pos(src);
                follow(w, src, t, band, *follow_tiles);
                let to = w.pos(src);
                if to == from {
                    return String::new();
                }
                format!("[Chase] {} closes in on {}: {} to {}", w.units[src].name, w.units[t].name, from, to)
            }
            CmdKind::Defend { grant_status, status_rounds, extra_grants, .. } => {
                if (grant_status.is_empty() && extra_grants.is_empty()) || !w.units[src].is_active() {
                    return String::new();
                }
                let name = w.units[src].name.clone();
                let u = &mut w.units[src];
                if !grant_status.is_empty() {
                    if status::defend_buffs().contains(&grant_status.as_str()) {
                        status::grant_now(u, grant_status, *status_rounds, self.status_magnitude, &name);
                    } else {
                        status::queue(u, grant_status, *status_rounds, self.status_magnitude, &name);
                    }
                }
                status::grant_extras(u, extra_grants, true, &name);
                format!("[Grant] {} readies {}", name, grant_status)
            }
            _ => String::new(),
        }
    }

    /// The command's effect, at the end of its last block.
    pub fn execute(&mut self, w: &mut World, rng: &mut GodotRng) {
        self.reactions.clear();
        self.broken_targets.clear();
        self.exploited.clear();
        self.failed = false;
        let src = self.source;
        let src_name = w.units[src].name.clone();
        match self.kind.clone() {
            CmdKind::Attack { target, skill_mult, use_offhand, status_rounds, min_range, max_range, .. } => {
                self.set_attack_result(0, false, None);
                if !w.units[src].is_active() {
                    self.failed = true;
                    self.log = format!("[Action Failed] {src_name} cannot act");
                    return;
                }
                let Some(t) = effective_target(w, src, target).filter(|t| w.units[*t].is_active()) else {
                    self.failed = true;
                    self.log = format!("[Attack Miss] {src_name}: target no longer in combat");
                    return;
                };
                let band = reach_band(w, src, min_range, max_range);
                if !in_reach(w, src, t, band) {
                    self.failed = true;
                    self.log = format!("[Attack Out of Range] {src_name} cannot reach {}", w.units[t].name);
                    return;
                }
                w.units[src].active_target = Some(t);
                let damaging = skill_mult > 0.0;
                let matched = if damaging { skill::exploit_match(&self.exploit, &w.units[t], &[]) } else { (String::new(), 1.0) };
                let mut taken = 0;
                let mut crit = false;
                if damaging {
                    let ranged = w.units[src].attack_range > 1 && !use_offhand;
                    let cover = cover_between(w, src, t, ranged);
                    let hit = roll_hit(w, rng, src, t, ranged, use_offhand, skill_mult * matched.1, cover);
                    taken = w.units[t].take_damage(hit.final_dmg, true);
                    crit = hit.is_crit;
                    self.set_attack_result(taken, crit, Some(t));
                }
                let mut detail = String::new();
                if !self.status_effect.is_empty() && w.units[t].is_alive() {
                    let ok = status::try_inflict(&mut w.units[t], &self.status_effect.clone(), self.status_potency, Some(rng), status_rounds, self.status_magnitude, &src_name);
                    detail = format!(" ({} {})", if ok { "inflicts" } else { "resisted" }, self.status_effect);
                }
                if !damaging {
                    self.log = format!("[Status] {src_name} -> {}{detail}", w.units[t].name);
                    return;
                }
                let ranged = w.units[src].attack_range > 1 && !use_offhand;
                if let Some(r) = riposte_counter(w, rng, src, t, ranged) {
                    self.reactions.push(r);
                }
                self.after_hit(w, t, taken, &matched);
                if !matched.0.is_empty() {
                    self.exploited.push((t, matched.0.clone()));
                }
                let tu = &w.units[t];
                self.log = format!("[Attack] {src_name} struck {} for {taken}{}{detail} (HP {}/{})", tu.name, if crit { " CRIT" } else { "" }, tu.hp, tu.max_hp);
            }
            CmdKind::Area { cells, skill_mult, status_rounds, .. } => {
                let mut total = 0;
                let mut hits = 0;
                let mut crits = Vec::new();
                if !w.units[src].is_active() {
                    self.failed = true;
                    self.log = format!("[Action Failed] {src_name} cannot act");
                    return;
                }
                let mut parts = Vec::new();
                for cell in cells {
                    let Some(v) = w.occupant(cell) else { continue };
                    if v == src || skill_mult <= 0.0 {
                        continue;
                    }
                    let ranged = w.units[src].attack_range > 1;
                    let cover = cover_for(&w.grid, w.pos(src), cell, ranged);
                    let matched = skill::exploit_match(&self.exploit, &w.units[v], &[]);
                    let hit = roll_hit(w, rng, src, v, ranged, false, skill_mult * matched.1, cover);
                    let taken = w.units[v].take_damage(hit.final_dmg, true);
                    total += taken;
                    hits += 1;
                    if hit.is_crit {
                        crits.push((v, !w.units[v].is_alive()));
                    }
                    if !self.status_effect.is_empty() && w.units[v].is_alive() {
                        status::try_inflict(&mut w.units[v], &self.status_effect.clone(), self.status_potency, Some(rng), status_rounds, self.status_magnitude, &src_name);
                    }
                    parts.push(format!("{} {taken}{}", w.units[v].name, if hit.is_crit { " CRIT" } else { "" }));
                    if let Some(r) = riposte_counter(w, rng, src, v, ranged) {
                        self.reactions.push(r);
                    }
                    self.after_hit(w, v, taken, &matched);
                    if !matched.0.is_empty() {
                        self.exploited.push((v, matched.0.clone()));
                    }
                }
                if let CmdKind::Area { last_damage, last_hits, crit_hits, .. } = &mut self.kind {
                    *last_damage = total;
                    *last_hits = hits;
                    *crit_hits = crits;
                }
                self.log = if hits == 0 { format!("[Area Miss] {src_name} hit nobody") } else { format!("[Attack] {src_name} area hit {}", parts.join(", ")) };
            }
            CmdKind::Defend { mitigation, guard_broken, .. } => {
                if !w.units[src].is_alive() {
                    self.failed = true;
                    return;
                }
                if guard_broken {
                    self.log = format!("[Defend] {src_name}'s guard was broken");
                    return;
                }
                w.units[src].guard = mitigation;
                self.log = if mitigation > 0.0 { format!("[Defend] {src_name} raises guard ({}% less damage)", (mitigation * 100.0).round() as i32) } else { format!("[Defend] {src_name} holds a defensive stance") };
            }
            CmdKind::Focus { boost, rounds } => {
                if !w.units[src].is_alive() {
                    return;
                }
                w.units[src].apply_attack_boost(boost, rounds);
                self.log = format!("[Focus] {src_name} x{boost} for {rounds} rounds");
            }
            CmdKind::Rest { recover_ap } => {
                if !w.units[src].is_alive() {
                    return;
                }
                let u = &mut w.units[src];
                u.ap = u.max_ap.min(u.ap + recover_ap);
                self.log = format!("[Rest] {src_name} AP {}/{}", u.ap, u.max_ap);
            }
            CmdKind::StatusBlock { status } => {
                self.log = format!("[Status] {src_name} is {status} and loses this block");
            }
            CmdKind::Support { target, skill, .. } => {
                if !w.units[src].is_active() || !w.units[target].is_active() {
                    self.failed = true;
                    self.log = format!("[Support Failed] {src_name}");
                    return;
                }
                if !support_in_reach(w, src, target, skill) {
                    self.failed = true;
                    self.log = format!("[Support Failed] {src_name} cannot reach {}", w.units[target].name);
                    return;
                }
                let healed = support_apply(w, src, target, skill);
                if let CmdKind::Support { last_heal, .. } = &mut self.kind {
                    *last_heal = healed;
                }
                self.log = format!("[Support] {src_name} used {} on {}: healed {healed}", self.name, w.units[target].name);
            }
            CmdKind::Item { target, item, .. } => {
                let problem = item_problem(w, src, target, &item);
                if !problem.is_empty() {
                    self.failed = true;
                    self.log = format!("[Item] {} not used: {problem}.", item.name);
                    return;
                }
                let idx = w.units[src].inventory.pouch.iter().position(|p| p.id == item.id).unwrap();
                w.units[src].inventory.pouch.remove(idx);
                let mut parts = Vec::new();
                let (mut heal, mut dmg, mut boost) = (0, 0, 0.0);
                let t = &mut w.units[target];
                if item.heal > 0 {
                    let before = t.hp;
                    t.heal(item.heal);
                    heal = t.hp - before;
                    parts.push(format!("healed {heal} (HP {}/{})", t.hp, t.max_hp));
                }
                if item.damage > 0 {
                    dmg = t.take_damage(item.damage, true);
                    parts.push(format!("{dmg} damage (HP {}/{}){}", t.hp, t.max_hp, if t.is_alive() { "" } else { ", defeated" }));
                }
                if item.buff_pct > 0.0 {
                    boost = 1.0 + item.buff_pct;
                    t.apply_attack_boost(boost, item.duration);
                    parts.push(format!("+{}% attack for {} rounds", roundi(item.buff_pct * 100.0), item.duration));
                }
                let who = if target == src { "themselves".to_string() } else { t.name.clone() };
                if let CmdKind::Item { used, last_heal, last_damage, last_boost, .. } = &mut self.kind {
                    (*used, *last_heal, *last_damage, *last_boost) = (true, heal, dmg, boost);
                }
                self.log = format!("[Item] {src_name} used {} on {who}: {}", item.name, if parts.is_empty() { "no effect".into() } else { parts.join(", ") });
            }
            CmdKind::Target { target } => {
                if !w.units[src].is_alive() {
                    return;
                }
                w.units[src].active_target = target;
                self.log = match target { Some(t) => format!("[Target] {src_name} locks on {}", w.units[t].name), None => format!("[Target] {src_name} clears its target") };
            }
            CmdKind::Walk { path, expected_start, goal, .. } => {
                let (moved, cut) = walk(w, rng, src, &path, expected_start, goal);
                if let CmdKind::Walk { moved_tiles, cut_short, .. } = &mut self.kind {
                    *moved_tiles = moved;
                    *cut_short = cut;
                }
                self.log = format!("[Move] {src_name} moved {moved} of {} tiles to {}", path.len(), w.pos(src));
            }
            CmdKind::Disengage { dest, max_tiles, .. } => {
                let ok = w.has(src) && !w.units[src].has_status("root") && w.pos(src).dist(dest) <= max_tiles && w.jump_unit(src, dest);
                if ok {
                    hazard_on_enter(w, rng, src, &[dest]);
                }
                if let CmdKind::Disengage { landed, .. } = &mut self.kind {
                    *landed = ok;
                }
                self.log = format!("[Disengage] {src_name} {} {}", if ok { "jumped to" } else { "could not jump to" }, dest);
            }
        }
    }

    fn set_attack_result(&mut self, dmg: i32, crit: bool, tgt: Option<UnitId>) {
        if let CmdKind::Attack { last_damage, last_crit, last_target, .. } = &mut self.kind {
            *last_damage = dmg;
            *last_crit = crit;
            *last_target = tgt;
        }
    }

    /// After one damaging hit: stance break (Offensive / Defensive target to Tactical), then exploit consume.
    fn after_hit(&mut self, w: &mut World, t: UnitId, taken: i32, matched: &(String, f64)) {
        let tu = &mut w.units[t];
        if self.disrupt_break && taken > 0 && tu.is_active() && matches!(tu.stance, Stance::Offensive | Stance::Defensive) {
            tu.set_stance(Stance::Tactical);
            tu.guard = 0.0;
            self.broken_targets.push(t);
        }
        if self.exploit_consume && !matched.0.is_empty() {
            status::remove_active(&mut w.units[t], &matched.0);
        }
    }
}

/// One hit with a crit roll drawn from the battle RNG (DamageCalculator.compute with the resolver's rng).
pub fn roll_hit(w: &World, rng: &mut GodotRng, a: UnitId, t: UnitId, ranged: bool, offhand: bool, skill_mult: f64, cover: f64) -> Hit {
    let roll = rng.randf();
    damage::compute(&w.units[a], &w.units[t], HitInput { ranged, offhand, skill_mult, crit_roll: Some(roll), cover, ..HitInput::default() })
}

fn effective_target(w: &World, src: UnitId, target: Option<UnitId>) -> Option<UnitId> {
    target.or(w.units[src].active_target)
}

/// The attack's reach band: the command's own, else the unit's (ranged 2..range, melee 1).
fn reach_band(w: &World, src: UnitId, min_range: i32, max_range: i32) -> (i32, i32) {
    let u = &w.units[src];
    (if min_range > 0 { min_range } else { u.min_attack_range() }, if max_range > 0 { max_range } else { u.attack_range })
}

fn in_reach(w: &World, src: UnitId, t: UnitId, band: (i32, i32)) -> bool {
    if !w.has(src) || !w.has(t) {
        return false;
    }
    let d = w.dist(src, t);
    d >= band.0 && d <= band.1
}

/// Cells around `target` within the band that a unit could stand on (walkable, not an exit, not blocked).
fn band_goals(w: &World, target_pos: V2, band: (i32, i32), blocked: &Cells) -> Cells {
    let mut goals = Cells::new();
    let hi = band.1;
    for dx in -hi..=hi {
        for dy in -hi..=hi {
            let cell = target_pos + V2::new(dx, dy);
            let d = dx.abs() + dy.abs();
            if d >= band.0 && d <= hi && w.grid.is_valid(cell) && w.grid.is_walkable(cell) && !w.grid.is_exit(cell) && !blocked.contains(&cell) {
                goals.insert(cell);
            }
        }
    }
    goals
}

fn others_blocked(w: &World, src: UnitId, active_only: bool) -> Cells {
    let mut blocked = Cells::new();
    for (u, p) in w.grid.positions.iter().enumerate() {
        if let Some(p) = p {
            if u != src && (!active_only || w.units[u].is_active()) {
                blocked.insert(*p);
            }
        }
    }
    blocked
}

/// Gap-close: route to the nearest tile in reach within `dash_tiles` cost; hazards entered roll their status.
fn dash(w: &mut World, rng: &mut GodotRng, src: UnitId, tgt: Option<UnitId>, band: (i32, i32), dash_tiles: i32) -> String {
    let Some(t) = tgt else { return String::new() };
    if !w.units[t].is_active() || !w.units[src].is_active() || in_reach(w, src, t, band) {
        return String::new();
    }
    let name = w.units[src].name.clone();
    if w.units[src].has_status("root") {
        return format!("[Dash Blocked] {name} is rooted");
    }
    let from = w.pos(src);
    let blocked = others_blocked(w, src, true);
    let goals = band_goals(w, w.pos(t), band, &blocked);
    let route = path::find_path(&w.grid, from, &goals, &blocked, false, &Hooks::default());
    if route.is_empty() || !dash_fits(&w.grid, from, &route, dash_tiles) {
        return format!("[Dash Blocked] {name} can't reach {}", w.units[t].name);
    }
    let mut entered = Vec::new();
    for d in route {
        if !w.move_unit(src, d) {
            break;
        }
        entered.push(w.pos(src));
    }
    let to = w.pos(src);
    if to == from {
        return format!("[Dash Blocked] {name} couldn't move");
    }
    hazard_on_enter(w, rng, src, &entered);
    format!("[Dash] {name} dashes: {from} to {to}")
}

pub fn dash_fits(grid: &Grid, from: V2, route: &[V2], dash_tiles: i32) -> bool {
    dash_tiles > 0 && path::blocks_needed(grid, from, route, dash_tiles) <= 1
}

/// Steps toward a target that moved out of reach, 1 AP per tile (AP debt allowed).
fn follow(w: &mut World, src: UnitId, t: UnitId, band: (i32, i32), follow_tiles: i32) {
    let mut left = follow_tiles;
    while left > 0 && !in_reach(w, src, t, band) {
        let blocked = others_blocked(w, src, false);
        let goals = band_goals(w, w.pos(t), band, &blocked);
        let route = path::find_path(&w.grid, w.pos(src), &goals, &blocked, false, &Hooks::default());
        if route.is_empty() || !w.move_unit(src, route[0]) {
            return;
        }
        w.units[src].ap -= 1;
        left -= 1;
    }
}

/// WalkCommand.execute: follows the planned steps (re-paths when not where the plan expected); unwalked tiles are
/// refunded as AP. Returns (tiles moved, cut short).
fn walk(w: &mut World, rng: &mut GodotRng, src: UnitId, planned: &[V2], expected_start: V2, goal: V2) -> (i32, bool) {
    if !w.has(src) {
        return (0, true);
    }
    let start = w.pos(src);
    let route: Vec<V2> = if expected_start != NO_CELL && goal != NO_CELL && start != expected_start {
        let blocked = others_blocked(w, src, false);
        let goals: Cells = [goal].into_iter().collect();
        let full = path::find_path(&w.grid, start, &goals, &blocked, w.grid.is_exit(goal), &Hooks::default());
        let budget = w.units[src].move_per_block().max(path::path_cost(&w.grid, expected_start, planned));
        path::fit_blocks(&w.grid, start, &full, budget, 1).into_iter().take(planned.len()).collect()
    } else {
        planned.to_vec()
    };
    let mut moved = 0;
    let mut cut = false;
    let mut entered = Vec::new();
    for d in route {
        let next = w.pos(src) + d;
        if !w.units[src].is_active() || !w.move_unit(src, d) {
            cut = true;
            break;
        }
        moved += 1;
        if w.grid.pos(src).is_some() {
            entered.push(next);
        }
    }
    hazard_on_enter(w, rng, src, &entered);
    if w.units[src].is_withdrawn {
        return (moved, cut);
    }
    if (moved as usize) < planned.len() {
        let u = &mut w.units[src];
        u.ap = u.max_ap.min(u.ap + (planned.len() as i32 - moved));
        cut = true;
    }
    (moved, cut)
}

/// CoverRules.cover_for: the target tile's cover, or a blocking neighbour toward the attacker (ranged hits only).
pub fn cover_for(grid: &Grid, attacker: V2, target: V2, ranged: bool) -> f64 {
    if !ranged || !grid.is_valid(target) {
        return 0.0;
    }
    let mut best = grid.cover_guard(target);
    let dx = (attacker.x - target.x).signum();
    let dy = (attacker.y - target.y).signum();
    let cells: Vec<V2> = if dx == 0 && dy == 0 {
        vec![]
    } else if dx == 0 || dy == 0 {
        vec![target + V2::new(dx, dy)]
    } else {
        vec![target + V2::new(dx, 0), target + V2::new(0, dy)]
    };
    for c in cells {
        if grid.is_valid(c) && !grid.is_walkable(c) {
            best = best.max(grid.cover_guard(c));
        }
    }
    best
}

pub fn cover_between(w: &World, a: UnitId, t: UnitId, ranged: bool) -> f64 {
    if !w.has(a) || !w.has(t) {
        return 0.0;
    }
    cover_for(&w.grid, w.pos(a), w.pos(t), ranged)
}

/// HazardRules.on_enter: each enter-hazard status of the entered cells rolls once (the first tile of a status wins).
pub fn hazard_on_enter(w: &mut World, rng: &mut GodotRng, u: UnitId, cells: &[V2]) {
    if !w.units[u].is_active() {
        return;
    }
    let mut seen: Vec<String> = Vec::new();
    for &cell in cells {
        let Some(h) = w.grid.hazard_at(cell).cloned() else { continue };
        if !(h.on == "enter" || h.on == "both") || seen.contains(&h.status) {
            continue;
        }
        seen.push(h.status.clone());
        status::try_inflict(&mut w.units[u], &h.status, h.potency, Some(rng), h.rounds, h.magnitude, "terrain");
    }
}

/// HazardRules.at_round_end: every active unit on a round-end hazard tile rolls it, in `units` order.
pub fn hazard_round_end(w: &mut World, rng: &mut GodotRng, units: &[UnitId]) {
    for &u in units {
        if !w.units[u].is_active() || !w.has(u) {
            continue;
        }
        let Some(h) = w.grid.hazard_at(w.pos(u)).cloned() else { continue };
        if h.on == "round_end" || h.on == "both" {
            status::try_inflict(&mut w.units[u], &h.status, h.potency, Some(rng), h.rounds, h.magnitude, "terrain");
        }
    }
}

/// RiposteRules.counter: a riposting defender answers an adjacent melee hit it survived, once per attacker per round.
pub fn riposte_counter(w: &mut World, rng: &mut GodotRng, attacker: UnitId, defender: UnitId, ranged: bool) -> Option<Reaction> {
    if ranged || attacker == defender {
        return None;
    }
    let d = &w.units[defender];
    if !d.has_status("riposte") || !d.is_alive() || !w.units[attacker].is_active() || d.riposted.contains(&attacker) {
        return None;
    }
    if !w.has(attacker) || !w.has(defender) || w.dist(attacker, defender) != 1 {
        return None;
    }
    w.units[defender].riposted.push(attacker);
    let du = &w.units[defender];
    let mult = COUNTER_MULT * du.weaken_factor() * du.empower_factor();
    let hit = roll_hit(w, rng, defender, attacker, false, false, mult, 0.0);
    let taken = w.units[attacker].take_damage(hit.final_dmg, true);
    Some(Reaction { defender, attacker, damage: taken, crit: hit.is_crit, defeated: !w.units[attacker].is_alive() })
}

// ---- SupportRules ---------------------------------------------------------------------------------------------

pub fn heal_amount(w: &World, caster: UnitId, skill_id: SkillId) -> i32 {
    let s = &w.cat.skills[skill_id];
    if !skill::heals(s) {
        return 0;
    }
    let c = &w.units[caster];
    let stat = if s.heal_stat.is_empty() { 0 } else { c.stats.stat(&s.heal_stat, c.weapon_context().as_deref()) };
    let base = s.heal_base + roundi(stat as f64 * s.heal_mult);
    roundi(base as f64 * c.potency_for_skill(s)).max(0)
}

pub fn effective_heal(target: &Unit, amount: i32) -> i32 {
    if !target.is_alive() {
        return 0;
    }
    amount.clamp(0, target.max_hp - target.hp)
}

pub fn cleansable(target: &Unit, s: &aegis_data::SkillDef) -> Vec<&'static str> {
    skill::cleanse_list(s).into_iter().filter(|st| target.st.has_any(st)).collect()
}

/// "" when `target` is legal for `skill` cast by `caster` (distance aside).
pub fn target_problem(w: &World, caster: UnitId, skill_id: SkillId, target: Option<UnitId>) -> String {
    let s = &w.cat.skills[skill_id];
    let Some(t) = target.filter(|t| w.units[*t].is_active()) else { return "Choose a friendly unit.".into() };
    if t == caster {
        return if s.target_mode == "ALLY" { "only on another ally".into() } else { String::new() };
    }
    if s.target_mode == "SELF" {
        return "self only".into();
    }
    if !w.friends_of(caster).contains(&t) {
        return "not an ally".into();
    }
    String::new()
}

// ---- ItemCommand shared rules (planner, AI, command) --------------------------------------------------------

/// ITEMUSE1 rule 3: every item use costs 1 AP (AP debt allowed).
pub const ITEM_AP: i32 = 1;

/// ItemCommand.pouch_item: the first consumable with `item_id` in the unit's battle pouch.
pub fn pouch_item<'a>(w: &'a World, c: UnitId, item_id: &str) -> Option<&'a ItemDef> {
    w.units[c].inventory.pouch.iter().find(|p| p.id == item_id)
}

pub fn pouch_count(w: &World, c: UnitId, item_id: &str) -> i32 {
    w.units[c].inventory.pouch.iter().filter(|p| p.id == item_id).count() as i32
}

/// Distinct pouch item ids in slot order.
pub fn pouch_ids(w: &World, c: UnitId) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for p in &w.units[c].inventory.pouch {
        if !out.contains(&p.id) {
            out.push(p.id.clone());
        }
    }
    out
}

/// ItemCommand.side_problem: "" when `user` may aim `item` at `target` by side (and reach 0 = self only).
pub fn item_side_problem(w: &World, user: UnitId, item: &ItemDef, target: Option<UnitId>) -> String {
    let Some(target) = target else { return format!("Pick a target for {}.", item.name) };
    let foe = w.foes_of(user).contains(&target);
    if item.aims_at_foes() {
        if !foe {
            return format!("{} is thrown at a foe.", item.name);
        }
    } else if foe {
        return format!("{} is used on yourself or an ally.", item.name);
    }
    if item.battle_reach() == 0 && target != user {
        return format!("{} is used on yourself only.", item.name);
    }
    String::new()
}

/// ItemCommand.problem: why the item can't be used now ("" = it can).
fn item_problem(w: &World, src: UnitId, target: UnitId, item: &ItemDef) -> String {
    if !w.units[src].is_active() {
        return format!("{} can't act", w.units[src].name);
    }
    if !w.units[target].is_active() {
        return format!("{} is down", w.units[target].name);
    }
    let in_reach = target == src || (w.has(src) && w.has(target) && w.dist(src, target) <= item.battle_reach());
    if !in_reach {
        return format!("{} is out of reach", w.units[target].name);
    }
    if pouch_item(w, src, &item.id).is_none() {
        return format!("none left in {}'s pouch", w.units[src].name);
    }
    String::new()
}

fn support_in_reach(w: &World, src: UnitId, target: UnitId, skill_id: SkillId) -> bool {
    if target == src {
        return true;
    }
    if !w.has(src) || !w.has(target) {
        return false;
    }
    let (lo, hi) = skill::support_reach(&w.cat.skills[skill_id], &w.units[src]);
    let d = w.dist(src, target);
    d >= lo && d <= hi
}

/// SupportRules.apply: cleanse, heal, grant, extra grants. Returns the HP healed.
fn support_apply(w: &mut World, caster: UnitId, target: UnitId, skill_id: SkillId) -> i32 {
    let s = w.cat.skills[skill_id].clone();
    for st in skill::cleanse_list(&s) {
        status::remove(&mut w.units[target], st);
    }
    let mut healed = 0;
    if skill::heals(&s) {
        let amount = heal_amount(w, caster, skill_id);
        let before = w.units[target].hp;
        w.units[target].heal(amount);
        healed = w.units[target].hp - before;
    }
    let name = w.units[caster].name.clone();
    if !s.grant_status.is_empty() {
        status::queue(&mut w.units[target], &s.grant_status, s.status_rounds, s.status_magnitude, &name);
    }
    status::grant_extras(&mut w.units[target], &s.extra_grants, false, &name);
    healed
}
