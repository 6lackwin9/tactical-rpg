//! CombatResolver (scripts/combat/arena/combat_resolver.gd): the round timeline. Commands pay AP and enter their
//! stance when they START; their effect lands at the end of their last block, highest initiative first.

use crate::command::{hazard_round_end, CmdKind, Command};
use crate::world::{UnitId, World};
use aegis_core::rng::GodotRng;
use aegis_core::sort::godot_sort;
use aegis_rules::status;

#[derive(Debug, Clone)]
pub struct ScheduledAction {
    pub unit: UnitId,
    pub command: Command,
    pub start: i32,
    pub end: i32,
    pub cancelled: bool,
}

#[derive(Debug, Clone)]
pub struct Resolver {
    pub scheduled: Vec<ScheduledAction>,
    pub registered: Vec<UnitId>,
    pub current_round: i32,
    pub total_blocks: i32,
    pub rng: GodotRng,
    pub log: Vec<String>,
}

impl Resolver {
    pub fn new(total_blocks: i32, seed: i64) -> Self {
        Self { scheduled: Vec::new(), registered: Vec::new(), current_round: 1, total_blocks, rng: GodotRng::from_i64(seed), log: Vec::new() }
    }

    pub fn register(&mut self, u: UnitId) {
        if !self.registered.contains(&u) {
            self.registered.push(u);
        }
    }

    pub fn schedule(&mut self, unit: UnitId, command: Command, start: i32) -> bool {
        self.register(unit);
        let end = start + command.block_cost - 1;
        if start < 1 || end > self.total_blocks {
            return false;
        }
        if self.scheduled.iter().any(|a| a.unit == unit && !a.cancelled && !(end < a.start || start > a.end)) {
            return false;
        }
        self.scheduled.push(ScheduledAction { unit, command, start, end, cancelled: false });
        true
    }

    /// Resets block guards, starts the commands that begin now, raises running guards, and returns the indices of
    /// the actions that resolve this block, highest initiative first.
    pub fn prepare_block(&mut self, w: &mut World, block: i32) -> Vec<usize> {
        for &c in &self.registered {
            w.units[c].guard = 0.0;
        }
        self.begin_actions(w, block);
        for a in &self.scheduled {
            if !a.cancelled && w.units[a.unit].is_active() && block >= a.start && block <= a.end {
                if let CmdKind::Defend { mitigation, guard_broken, .. } = &a.command.kind {
                    if !guard_broken {
                        w.units[a.unit].guard = *mitigation;
                    }
                }
            }
        }
        let mut resolving: Vec<usize> = (0..self.scheduled.len()).filter(|&i| !self.scheduled[i].cancelled && self.scheduled[i].end == block).collect();
        let inits: Vec<i32> = (0..self.scheduled.len()).map(|i| self.scheduled[i].command.initiative(w)).collect();
        godot_sort(&mut resolving, |a, b| inits[*a] > inits[*b]);
        resolving
    }

    /// Commands starting now pay AP (skill + stance switch; debt allowed), enter their stance and run on_start.
    fn begin_actions(&mut self, w: &mut World, block: i32) {
        for i in 0..self.scheduled.len() {
            let a = &self.scheduled[i];
            if a.cancelled || a.start != block || !w.units[a.unit].is_active() {
                continue;
            }
            let c = a.unit;
            let total = a.command.ap_cost + a.command.stance_switch_cost;
            w.units[c].ap -= total;
            if let Some(s) = a.command.stance_on_start {
                if s != w.units[c].stance {
                    w.units[c].set_stance(s);
                }
            }
            let msg = self.scheduled[i].command.on_start(w, &mut self.rng);
            if !msg.is_empty() {
                self.log.push(msg);
            }
        }
    }

    pub fn execute_single(&mut self, w: &mut World, idx: usize) {
        if self.scheduled[idx].cancelled {
            return;
        }
        let unit = self.scheduled[idx].unit;
        if !w.units[unit].is_active() {
            self.scheduled[idx].cancelled = true;
            return;
        }
        self.scheduled[idx].command.execute(w, &mut self.rng);
        let cmd = &self.scheduled[idx].command;
        if !cmd.log.is_empty() {
            self.log.push(cmd.log.clone());
        }
        self.apply_combo(w, idx);
        let inactive: Vec<UnitId> = self.registered.iter().copied().filter(|c| !w.units[*c].is_active()).collect();
        for c in inactive {
            for a in self.scheduled.iter_mut() {
                if a.unit == c {
                    a.cancelled = true;
                }
            }
        }
    }

    /// Stance breaks end the broken units' running guards; a flow move then enters its end stance for free.
    fn apply_combo(&mut self, w: &mut World, idx: usize) {
        let end = self.scheduled[idx].end;
        let broken = self.scheduled[idx].command.broken_targets.clone();
        for b in broken {
            for other in self.scheduled.iter_mut() {
                let running = other.start <= end && other.end >= end;
                if other.unit == b && running && !other.cancelled {
                    if let CmdKind::Defend { guard_broken, .. } = &mut other.command.kind {
                        *guard_broken = true;
                    }
                }
            }
        }
        let a = &self.scheduled[idx];
        if let Some(s) = a.command.stance_on_end {
            if !a.command.failed && w.units[a.unit].is_active() {
                w.units[a.unit].set_stance(s);
            }
        }
    }

    /// Round end per unit in registered order: bleed, poison, burn, regen; then round-end hazards.
    pub fn apply_round_end_effects(&mut self, w: &mut World) {
        for &c in &self.registered.clone() {
            if w.units[c].is_active() && w.units[c].has_status("bleed") {
                let dmg = status::bleed_damage(&w.units[c]);
                w.units[c].take_damage(dmg, true);
            }
            if w.units[c].is_active() && w.units[c].has_status("poison") && status::poison_damage(&w.units[c]) > 0 {
                let dmg = status::poison_damage(&w.units[c]);
                w.units[c].take_damage(dmg, true);
            }
            if w.units[c].is_active() && w.units[c].has_status("burn") {
                let dmg = status::burn_damage(&w.units[c]);
                w.units[c].take_damage(dmg, true);
            }
            if w.units[c].is_active() && w.units[c].has_status("regen") {
                let u = &mut w.units[c];
                let amount = status::regen_amount(u).min(u.max_hp - u.hp);
                u.heal(amount);
            }
        }
        let units = self.registered.clone();
        hazard_round_end(w, &mut self.rng, &units);
    }

    pub fn finish_round(&mut self, w: &mut World) {
        for &c in &self.registered {
            if w.units[c].is_active() {
                w.units[c].tick_round_buffs();
            }
        }
        self.scheduled.clear();
        self.current_round += 1;
    }
}
